#[cfg(feature = "ai_zkp")]
mod ai_zkp_tests {
    use lattice_execution::tensor::{Tensor, TensorEngine, TensorOps};
    use lattice_execution::zkp::{ProofType, ZKPBackend};

    #[test]
    fn test_tensor_operations() {
        let mut engine = TensorEngine::new(100); // 100MB max memory
        let shape = vec![2, 3];
        let data_a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let data_b = vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let tensor_a_id = engine.create_tensor(data_a, shape.clone()).unwrap();
        let tensor_b_id = engine.create_tensor(data_b, shape.clone()).unwrap();
        let result_id = engine.add(&tensor_a_id, &tensor_b_id).unwrap();
        assert!(engine.get_tensor(&result_id).is_some());
        let mul_result_id = engine.mul(&tensor_a_id, &tensor_b_id).unwrap();
        assert!(engine.get_tensor(&mul_result_id).is_some());
        engine.delete_tensor(&tensor_a_id).unwrap();
        engine.delete_tensor(&tensor_b_id).unwrap();
    }

    #[test]
    fn test_tensor_activations() {
        let tensor = Tensor::new(vec![-1.0, 0.0, 1.0, 2.0], vec![2, 2]).unwrap();
        let relu_result = TensorOps::relu(&tensor);
        assert_eq!(relu_result.shape.0, vec![2, 2]);
        let sigmoid_result = TensorOps::sigmoid(&tensor);
        assert_eq!(sigmoid_result.shape.0, vec![2, 2]);
        let tanh_result = TensorOps::tanh(&tensor);
        assert_eq!(tanh_result.shape.0, vec![2, 2]);
    }

    #[test]
    fn test_zkp_backend() {
        let backend = ZKPBackend::new();
        backend.initialize().unwrap();
        let estimate = backend.estimate_proving_time(ProofType::ModelExecution);
        assert!(estimate > 0);
        let proof = backend
            .prove_tensor_computation("add", vec![vec![1, 2, 3], vec![4, 5, 6]], vec![5, 7, 9])
            .unwrap();
        assert!(!proof.proof_bytes.is_empty());
        assert!(!proof.public_inputs.is_empty());
    }

    #[test]
    fn test_vm_integration() {
        use lattice_execution::vm::ai_opcodes::AIOpcode;
        use lattice_execution::vm::VM;
        let mut vm = VM::new(1_000_000);
        let opcodes = vec![
            (0xA0, Some(AIOpcode::LOAD_MODEL)),
            (0xA1, Some(AIOpcode::UNLOAD_MODEL)),
            (0xB0, Some(AIOpcode::TENSOR_NEW)),
            (0xB1, Some(AIOpcode::TENSOR_ADD)),
            (0xD0, Some(AIOpcode::VERIFY_PROOF)),
        ];
        for (opcode_byte, expected) in opcodes {
            let bytecode = vec![opcode_byte, 0x00];
            let result = vm.execute(&bytecode);
            if let Err(e) = result {
                if let lattice_execution::ExecutionError::InvalidOpcode(_) = e {
                    if expected.is_some() {
                        panic!("Valid opcode {:02x} treated as invalid", opcode_byte);
                    }
                }
            }
        }
    }
}
