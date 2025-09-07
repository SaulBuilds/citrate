-- CreateTable
CREATE TABLE "Block" (
    "id" TEXT NOT NULL,
    "number" BIGINT NOT NULL,
    "hash" TEXT NOT NULL,
    "parentHash" TEXT NOT NULL,
    "selectedParent" TEXT,
    "mergeParents" TEXT[],
    "timestamp" TIMESTAMP(3) NOT NULL,
    "miner" TEXT NOT NULL,
    "gasUsed" BIGINT NOT NULL,
    "gasLimit" BIGINT NOT NULL,
    "blueScore" BIGINT,
    "isBlue" BOOLEAN NOT NULL DEFAULT true,
    "difficulty" BIGINT,
    "totalDifficulty" BIGINT,
    "size" INTEGER NOT NULL,
    "stateRoot" TEXT NOT NULL,
    "txRoot" TEXT NOT NULL,
    "receiptRoot" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Block_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Transaction" (
    "id" TEXT NOT NULL,
    "hash" TEXT NOT NULL,
    "blockHash" TEXT NOT NULL,
    "blockNumber" BIGINT NOT NULL,
    "from" TEXT NOT NULL,
    "to" TEXT,
    "value" TEXT NOT NULL,
    "gas" BIGINT NOT NULL,
    "gasPrice" TEXT NOT NULL,
    "nonce" BIGINT NOT NULL,
    "data" TEXT,
    "status" BOOLEAN NOT NULL,
    "contractAddress" TEXT,
    "type" TEXT,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Transaction_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Log" (
    "id" TEXT NOT NULL,
    "transactionHash" TEXT NOT NULL,
    "logIndex" INTEGER NOT NULL,
    "address" TEXT NOT NULL,
    "topics" TEXT[],
    "data" TEXT NOT NULL,
    "eventName" TEXT,

    CONSTRAINT "Log_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Model" (
    "id" TEXT NOT NULL,
    "modelId" TEXT NOT NULL,
    "owner" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "version" TEXT NOT NULL,
    "format" TEXT NOT NULL,
    "dataHash" TEXT NOT NULL,
    "metadata" JSONB NOT NULL,
    "blockNumber" BIGINT NOT NULL,
    "blockHash" TEXT NOT NULL,
    "transactionHash" TEXT NOT NULL,
    "timestamp" TIMESTAMP(3) NOT NULL,
    "size" BIGINT,
    "permissions" TEXT[],
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Model_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "ModelOperation" (
    "id" TEXT NOT NULL,
    "modelId" TEXT NOT NULL,
    "operationType" TEXT NOT NULL,
    "transactionHash" TEXT NOT NULL,
    "blockNumber" BIGINT NOT NULL,
    "timestamp" TIMESTAMP(3) NOT NULL,
    "details" JSONB,

    CONSTRAINT "ModelOperation_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Inference" (
    "id" TEXT NOT NULL,
    "inferenceId" TEXT NOT NULL,
    "modelId" TEXT NOT NULL,
    "executorAddress" TEXT NOT NULL,
    "inputHash" TEXT NOT NULL,
    "outputHash" TEXT NOT NULL,
    "proofId" TEXT,
    "gasUsed" BIGINT NOT NULL,
    "executionTime" INTEGER NOT NULL,
    "timestamp" TIMESTAMP(3) NOT NULL,
    "blockNumber" BIGINT NOT NULL,
    "transactionHash" TEXT NOT NULL,

    CONSTRAINT "Inference_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Account" (
    "id" TEXT NOT NULL,
    "address" TEXT NOT NULL,
    "balance" TEXT NOT NULL,
    "nonce" BIGINT NOT NULL,
    "isContract" BOOLEAN NOT NULL DEFAULT false,
    "isModel" BOOLEAN NOT NULL DEFAULT false,
    "firstSeen" TIMESTAMP(3) NOT NULL,
    "lastSeen" TIMESTAMP(3) NOT NULL,
    "transactionCount" INTEGER NOT NULL DEFAULT 0,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Account_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "DagStats" (
    "id" TEXT NOT NULL,
    "timestamp" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "totalBlocks" BIGINT NOT NULL,
    "blueBlocks" BIGINT NOT NULL,
    "redBlocks" BIGINT NOT NULL,
    "tipsCount" INTEGER NOT NULL,
    "maxBlueScore" BIGINT NOT NULL,
    "avgBlockTime" DOUBLE PRECISION NOT NULL,
    "tps" DOUBLE PRECISION NOT NULL,
    "activeModels" INTEGER NOT NULL,
    "totalInferences" BIGINT NOT NULL,

    CONSTRAINT "DagStats_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "SearchIndex" (
    "id" TEXT NOT NULL,
    "type" TEXT NOT NULL,
    "identifier" TEXT NOT NULL,
    "data" JSONB NOT NULL,

    CONSTRAINT "SearchIndex_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "Block_number_key" ON "Block"("number");

-- CreateIndex
CREATE UNIQUE INDEX "Block_hash_key" ON "Block"("hash");

-- CreateIndex
CREATE INDEX "Block_number_idx" ON "Block"("number");

-- CreateIndex
CREATE INDEX "Block_hash_idx" ON "Block"("hash");

-- CreateIndex
CREATE INDEX "Block_timestamp_idx" ON "Block"("timestamp");

-- CreateIndex
CREATE INDEX "Block_miner_idx" ON "Block"("miner");

-- CreateIndex
CREATE INDEX "Block_blueScore_idx" ON "Block"("blueScore");

-- CreateIndex
CREATE UNIQUE INDEX "Transaction_hash_key" ON "Transaction"("hash");

-- CreateIndex
CREATE INDEX "Transaction_hash_idx" ON "Transaction"("hash");

-- CreateIndex
CREATE INDEX "Transaction_from_idx" ON "Transaction"("from");

-- CreateIndex
CREATE INDEX "Transaction_to_idx" ON "Transaction"("to");

-- CreateIndex
CREATE INDEX "Transaction_blockNumber_idx" ON "Transaction"("blockNumber");

-- CreateIndex
CREATE INDEX "Transaction_type_idx" ON "Transaction"("type");

-- CreateIndex
CREATE INDEX "Log_transactionHash_idx" ON "Log"("transactionHash");

-- CreateIndex
CREATE INDEX "Log_address_idx" ON "Log"("address");

-- CreateIndex
CREATE INDEX "Log_eventName_idx" ON "Log"("eventName");

-- CreateIndex
CREATE UNIQUE INDEX "Model_modelId_key" ON "Model"("modelId");

-- CreateIndex
CREATE INDEX "Model_modelId_idx" ON "Model"("modelId");

-- CreateIndex
CREATE INDEX "Model_owner_idx" ON "Model"("owner");

-- CreateIndex
CREATE INDEX "Model_name_idx" ON "Model"("name");

-- CreateIndex
CREATE INDEX "Model_format_idx" ON "Model"("format");

-- CreateIndex
CREATE INDEX "ModelOperation_modelId_idx" ON "ModelOperation"("modelId");

-- CreateIndex
CREATE INDEX "ModelOperation_operationType_idx" ON "ModelOperation"("operationType");

-- CreateIndex
CREATE INDEX "ModelOperation_timestamp_idx" ON "ModelOperation"("timestamp");

-- CreateIndex
CREATE UNIQUE INDEX "Inference_inferenceId_key" ON "Inference"("inferenceId");

-- CreateIndex
CREATE INDEX "Inference_inferenceId_idx" ON "Inference"("inferenceId");

-- CreateIndex
CREATE INDEX "Inference_modelId_idx" ON "Inference"("modelId");

-- CreateIndex
CREATE INDEX "Inference_executorAddress_idx" ON "Inference"("executorAddress");

-- CreateIndex
CREATE INDEX "Inference_timestamp_idx" ON "Inference"("timestamp");

-- CreateIndex
CREATE UNIQUE INDEX "Account_address_key" ON "Account"("address");

-- CreateIndex
CREATE INDEX "Account_address_idx" ON "Account"("address");

-- CreateIndex
CREATE INDEX "Account_isContract_idx" ON "Account"("isContract");

-- CreateIndex
CREATE INDEX "Account_isModel_idx" ON "Account"("isModel");

-- CreateIndex
CREATE INDEX "DagStats_timestamp_idx" ON "DagStats"("timestamp");

-- CreateIndex
CREATE INDEX "SearchIndex_type_idx" ON "SearchIndex"("type");

-- CreateIndex
CREATE INDEX "SearchIndex_identifier_idx" ON "SearchIndex"("identifier");

-- CreateIndex
CREATE UNIQUE INDEX "SearchIndex_type_identifier_key" ON "SearchIndex"("type", "identifier");

-- AddForeignKey
ALTER TABLE "Transaction" ADD CONSTRAINT "Transaction_blockHash_fkey" FOREIGN KEY ("blockHash") REFERENCES "Block"("hash") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Log" ADD CONSTRAINT "Log_transactionHash_fkey" FOREIGN KEY ("transactionHash") REFERENCES "Transaction"("hash") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Model" ADD CONSTRAINT "Model_blockHash_fkey" FOREIGN KEY ("blockHash") REFERENCES "Block"("hash") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "ModelOperation" ADD CONSTRAINT "ModelOperation_modelId_fkey" FOREIGN KEY ("modelId") REFERENCES "Model"("modelId") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "ModelOperation" ADD CONSTRAINT "ModelOperation_transactionHash_fkey" FOREIGN KEY ("transactionHash") REFERENCES "Transaction"("hash") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Inference" ADD CONSTRAINT "Inference_modelId_fkey" FOREIGN KEY ("modelId") REFERENCES "Model"("modelId") ON DELETE RESTRICT ON UPDATE CASCADE;
