pub mod ai_opcodes;

use crate::types::{ExecutionError, GasSchedule};
use primitive_types::U256;
use std::collections::HashMap;
use tracing::debug;

/// Virtual Machine for executing smart contracts and AI models
pub struct VM {
    pub stack: Stack,
    pub memory: Memory,
    pub storage: Storage,
    pub gas_remaining: u64,
    pub gas_used: u64,
    pub gas_schedule: GasSchedule,
    pub ai_extension: ai_opcodes::AIVMExtension,
}

impl VM {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            stack: Stack::new(),
            memory: Memory::new(),
            storage: Storage::new(),
            gas_remaining: gas_limit,
            gas_used: 0,
            gas_schedule: GasSchedule::default(),
            ai_extension: ai_opcodes::AIVMExtension::new(),
        }
    }
    
    /// Execute bytecode with input: seeds memory at offset 0 and pushes
    /// up to two 32-byte words from input onto the main stack (model_id, input_id).
    pub fn execute_with_input(&mut self, code: &[u8], input: &[u8]) -> Result<Vec<u8>, ExecutionError> {
        // Seed memory with input bytes
        if !input.is_empty() {
            self.memory.set(0, input)?;
        }
        // If at least 32 bytes, push model_id
        if input.len() >= 32 {
            let mut word = [0u8; 32];
            word.copy_from_slice(&input[0..32]);
            let model_id = U256::from_big_endian(&word);
            self.stack.push(model_id)?;
        }
        // If at least 64 bytes, push input_id on top
        if input.len() >= 64 {
            let mut word = [0u8; 32];
            word.copy_from_slice(&input[32..64]);
            let input_id = U256::from_big_endian(&word);
            self.stack.push(input_id)?;
        }
        
        let _ = self.execute(code)?;
        
        // If any value remains on the stack, return top as 32-byte big-endian
        if let Ok(val) = self.stack.pop() {
            let mut out = [0u8; 32];
            val.to_big_endian(&mut out);
            Ok(out.to_vec())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Execute bytecode
    pub fn execute(&mut self, code: &[u8]) -> Result<Vec<u8>, ExecutionError> {
        let mut pc = 0; // Program counter
        
        while pc < code.len() {
            let opcode = code[pc];
            
            // Check if it's an AI opcode
            if (0xA0..=0xDF).contains(&opcode) {
                if let Some(ai_opcode) = self.decode_ai_opcode(opcode) {
                    // Consume gas for AI opcode
                    let gas_cost = self.ai_extension.gas_cost(ai_opcode);
                    self.consume_gas(gas_cost)?;
                    
                    // Transfer required stack args to AI extension
                    let args_needed = ai_required_args(ai_opcode);
                    if args_needed > 0 {
                        let mut tmp: Vec<U256> = Vec::with_capacity(args_needed);
                        for _ in 0..args_needed {
                            let v = self.stack.pop()?; // error if insufficient
                            tmp.push(v);
                        }
                        // Preserve pop order expected by opcode: last pushed should be on top in ai stack
                        for v in tmp.into_iter().rev() {
                            self.ai_extension.push(v);
                        }
                    }
                    
                    // Execute AI opcode
                    self.ai_extension.execute(ai_opcode)?;
                    
                    // Transfer results back to main stack
                    if produces_output(ai_opcode) {
                        if let Some(value) = self.ai_extension.pop() {
                            self.stack.push(value)?;
                        }
                    }
                }
            } else {
                // Execute standard EVM opcodes
                self.execute_standard_opcode(opcode)?;
            }
            
            pc += 1;
        }
        
        Ok(vec![])
    }
    
    /// Decode AI opcode
    fn decode_ai_opcode(&self, opcode: u8) -> Option<ai_opcodes::AIOpcode> {
        use ai_opcodes::AIOpcode;
        
        match opcode {
            0xA0 => Some(AIOpcode::LOAD_MODEL),
            0xA1 => Some(AIOpcode::UNLOAD_MODEL),
            0xA2 => Some(AIOpcode::EXEC_MODEL),
            0xA3 => Some(AIOpcode::TRAIN_MODEL),
            
            0xB0 => Some(AIOpcode::TENSOR_NEW),
            0xB1 => Some(AIOpcode::TENSOR_ADD),
            0xB2 => Some(AIOpcode::TENSOR_MUL),
            0xB3 => Some(AIOpcode::TENSOR_MATMUL),
            
            0xD0 => Some(AIOpcode::VERIFY_PROOF),
            0xD1 => Some(AIOpcode::GENERATE_PROOF),
            
            _ => None,
        }
    }
    
    /// Execute standard opcode (simplified)
    fn execute_standard_opcode(&mut self, opcode: u8) -> Result<(), ExecutionError> {
        match opcode {
            // Stack operations
            0x50 => self.op_push()?,
            0x51 => self.op_pop()?,
            0x52 => self.op_mload()?,
            0x53 => self.op_mstore()?,
            
            // Arithmetic
            0x01 => self.op_add()?,
            0x02 => self.op_mul()?,
            0x03 => self.op_sub()?,
            0x04 => self.op_div()?,
            
            // Control flow
            0x56 => self.op_jump()?,
            0x57 => self.op_jumpi()?,
            0x00 => self.op_stop()?,
            
            _ => {
                debug!("Unknown opcode: 0x{:02x}", opcode);
                return Err(ExecutionError::InvalidOpcode(opcode));
            }
        }
        
        Ok(())
    }
    
    /// Consume gas
    pub fn consume_gas(&mut self, amount: u64) -> Result<(), ExecutionError> {
        if self.gas_remaining < amount {
            return Err(ExecutionError::OutOfGas);
        }
        
        self.gas_remaining -= amount;
        self.gas_used += amount;
        
        Ok(())
    }
    
    /// Get gas used
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }
    
    // Standard opcodes (simplified implementations)
    
    fn op_push(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.push)?;
        self.stack.push(U256::zero())?;
        Ok(())
    }
    
    fn op_pop(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.pop)?;
        self.stack.pop()?;
        Ok(())
    }
    
    fn op_mload(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.mload)?;
        let offset = self.stack.pop()?.as_u64() as usize;
        let value = self.memory.get_word(offset)?;
        self.stack.push(value)?;
        Ok(())
    }
    
    fn op_mstore(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.mstore)?;
        let offset = self.stack.pop()?.as_u64() as usize;
        let value = self.stack.pop()?;
        self.memory.set_word(offset, value)?;
        Ok(())
    }
    
    fn op_add(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.add)?;
        let a = self.stack.pop()?;
        let b = self.stack.pop()?;
        self.stack.push(a.overflowing_add(b).0)?;
        Ok(())
    }
    
    fn op_mul(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.mul)?;
        let a = self.stack.pop()?;
        let b = self.stack.pop()?;
        self.stack.push(a.overflowing_mul(b).0)?;
        Ok(())
    }
    
    fn op_sub(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.sub)?;
        let a = self.stack.pop()?;
        let b = self.stack.pop()?;
        self.stack.push(a.overflowing_sub(b).0)?;
        Ok(())
    }
    
    fn op_div(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.div)?;
        let a = self.stack.pop()?;
        let b = self.stack.pop()?;
        if b.is_zero() {
            self.stack.push(U256::zero())?;
        } else {
            self.stack.push(a / b)?;
        }
        Ok(())
    }
    
    fn op_jump(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.jump)?;
        let _dest = self.stack.pop()?;
        // Jump logic would go here
        Ok(())
    }
    
    fn op_jumpi(&mut self) -> Result<(), ExecutionError> {
        self.consume_gas(self.gas_schedule.jumpi)?;
        let _dest = self.stack.pop()?;
        let _cond = self.stack.pop()?;
        // Conditional jump logic would go here
        Ok(())
    }
    
    fn op_stop(&mut self) -> Result<(), ExecutionError> {
        // Stop execution
        Ok(())
    }
}

/// Check if opcode needs stack transfer
#[allow(dead_code)]
fn needs_stack_transfer(opcode: ai_opcodes::AIOpcode) -> bool {
    use ai_opcodes::AIOpcode;
    matches!(opcode, 
        AIOpcode::LOAD_MODEL | 
        AIOpcode::EXEC_MODEL | 
        AIOpcode::TENSOR_NEW | 
        AIOpcode::TENSOR_ADD |
        AIOpcode::TENSOR_MUL |
        AIOpcode::TENSOR_MATMUL |
        AIOpcode::VERIFY_PROOF |
        AIOpcode::GENERATE_PROOF
    )
}

/// Number of arguments an AI opcode expects to pop from the main stack
fn ai_required_args(opcode: ai_opcodes::AIOpcode) -> usize {
    use ai_opcodes::AIOpcode;
    match opcode {
        AIOpcode::LOAD_MODEL => 1,
        AIOpcode::UNLOAD_MODEL => 1,
        AIOpcode::EXEC_MODEL => 2,
        AIOpcode::TRAIN_MODEL => 2,
        AIOpcode::TENSOR_NEW => 3,     // dims, height, width
        AIOpcode::TENSOR_ADD => 2,
        AIOpcode::TENSOR_MUL => 2,
        AIOpcode::TENSOR_MATMUL => 2,
        AIOpcode::VERIFY_PROOF => 1,
        AIOpcode::GENERATE_PROOF => 1,
    }
}

/// Check if opcode produces output
fn produces_output(opcode: ai_opcodes::AIOpcode) -> bool {
    use ai_opcodes::AIOpcode;
    matches!(opcode,
        AIOpcode::LOAD_MODEL |
        AIOpcode::EXEC_MODEL |
        AIOpcode::TENSOR_NEW |
        AIOpcode::TENSOR_ADD |
        AIOpcode::TENSOR_MUL |
        AIOpcode::TENSOR_MATMUL |
        AIOpcode::VERIFY_PROOF |
        AIOpcode::GENERATE_PROOF
    )
}

/// Stack implementation
pub struct Stack {
    data: Vec<U256>,
    max_depth: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            max_depth: 1024,
        }
    }
    
    pub fn push(&mut self, value: U256) -> Result<(), ExecutionError> {
        if self.data.len() >= self.max_depth {
            return Err(ExecutionError::StackOverflow);
        }
        self.data.push(value);
        Ok(())
    }
    
    pub fn pop(&mut self) -> Result<U256, ExecutionError> {
        self.data.pop().ok_or(ExecutionError::StackUnderflow)
    }
}

impl Default for Stack {
    fn default() -> Self { Self::new() }
}

/// Memory implementation
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }
    
    pub fn get(&self, offset: usize, size: usize) -> Result<Vec<u8>, ExecutionError> {
        if offset + size > self.data.len() {
            return Ok(vec![0u8; size]); // Return zeros for uninitialized memory
        }
        Ok(self.data[offset..offset + size].to_vec())
    }
    
    pub fn set(&mut self, offset: usize, data: &[u8]) -> Result<(), ExecutionError> {
        let required_size = offset + data.len();
        if required_size > self.data.len() {
            self.data.resize(required_size, 0);
        }
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    
    pub fn get_word(&self, offset: usize) -> Result<U256, ExecutionError> {
        let bytes = self.get(offset, 32)?;
        Ok(U256::from_big_endian(&bytes))
    }
    
    pub fn set_word(&mut self, offset: usize, value: U256) -> Result<(), ExecutionError> {
        let mut bytes = [0u8; 32];
        value.to_big_endian(&mut bytes);
        self.set(offset, &bytes)
    }
}

impl Default for Memory {
    fn default() -> Self { Self::new() }
}

/// Storage implementation
pub struct Storage {
    data: HashMap<U256, U256>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn get(&self, key: &U256) -> U256 {
        self.data.get(key).copied().unwrap_or_default()
    }
    
    pub fn set(&mut self, key: U256, value: U256) {
        if value.is_zero() {
            self.data.remove(&key);
        } else {
            self.data.insert(key, value);
        }
    }
}

impl Default for Storage {
    fn default() -> Self { Self::new() }
}
