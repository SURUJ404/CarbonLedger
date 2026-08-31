-- CreateEnum
CREATE TYPE "ProjectStatus" AS ENUM ('PENDING', 'VERIFIED', 'SUSPENDED', 'REJECTED');

-- CreateEnum
CREATE TYPE "UserRole" AS ENUM ('DEVELOPER', 'CORPORATION', 'VERIFIER', 'ADMIN');

-- CreateTable
CREATE TABLE "User" (
    "id" TEXT NOT NULL,
    "stellarPubKey" TEXT NOT NULL,
    "email" TEXT,
    "role" "UserRole" NOT NULL DEFAULT 'DEVELOPER',
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "User_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Project" (
    "id" TEXT NOT NULL,
    "onChainId" BIGINT NOT NULL,
    "developerId" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "methodology" TEXT NOT NULL,
    "latMicro" BIGINT NOT NULL,
    "lngMicro" BIGINT NOT NULL,
    "methodologyScore" INTEGER,
    "status" "ProjectStatus" NOT NULL DEFAULT 'PENDING',
    "docsIpfsCid" TEXT,
    "lastMonitoringAt" TIMESTAMP(3),
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Project_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "CreditBatch" (
    "id" TEXT NOT NULL,
    "onChainId" BIGINT NOT NULL,
    "projectId" TEXT NOT NULL,
    "vintageYear" INTEGER NOT NULL,
    "serialStart" BIGINT NOT NULL,
    "serialEnd" BIGINT NOT NULL,
    "ownerPubKey" TEXT NOT NULL,
    "retired" BOOLEAN NOT NULL DEFAULT false,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "CreditBatch_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Listing" (
    "id" TEXT NOT NULL,
    "onChainId" BIGINT NOT NULL,
    "batchId" TEXT NOT NULL,
    "sellerId" TEXT NOT NULL,
    "pricePerTonne" BIGINT NOT NULL,
    "tonnes" BIGINT NOT NULL,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Listing_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Retirement" (
    "id" TEXT NOT NULL,
    "batchId" TEXT NOT NULL,
    "retiredById" TEXT NOT NULL,
    "beneficiary" TEXT NOT NULL,
    "reason" TEXT NOT NULL,
    "certificateUrl" TEXT NOT NULL,
    "tonnes" BIGINT NOT NULL,
    "retiredAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Retirement_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "MonitoringRecord" (
    "id" TEXT NOT NULL,
    "projectOnChainId" BIGINT NOT NULL,
    "dataHash" TEXT NOT NULL,
    "flagged" BOOLEAN NOT NULL DEFAULT false,
    "submittedAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "MonitoringRecord_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "BenchmarkPrice" (
    "id" TEXT NOT NULL,
    "methodology" TEXT NOT NULL,
    "vintageYear" INTEGER NOT NULL,
    "pricePerTonne" BIGINT NOT NULL,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "BenchmarkPrice_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "User_stellarPubKey_key" ON "User"("stellarPubKey");

-- CreateIndex
CREATE UNIQUE INDEX "User_email_key" ON "User"("email");

-- CreateIndex
CREATE UNIQUE INDEX "Project_onChainId_key" ON "Project"("onChainId");

-- CreateIndex
CREATE INDEX "Project_status_idx" ON "Project"("status");

-- CreateIndex
CREATE INDEX "Project_developerId_idx" ON "Project"("developerId");

-- CreateIndex
CREATE UNIQUE INDEX "CreditBatch_onChainId_key" ON "CreditBatch"("onChainId");

-- CreateIndex
CREATE INDEX "CreditBatch_projectId_idx" ON "CreditBatch"("projectId");

-- CreateIndex
CREATE INDEX "CreditBatch_serialStart_serialEnd_idx" ON "CreditBatch"("serialStart", "serialEnd");

-- CreateIndex
CREATE UNIQUE INDEX "Listing_onChainId_key" ON "Listing"("onChainId");

-- CreateIndex
CREATE INDEX "Listing_active_idx" ON "Listing"("active");

-- CreateIndex
CREATE INDEX "Listing_sellerId_idx" ON "Listing"("sellerId");

-- CreateIndex
CREATE UNIQUE INDEX "Retirement_batchId_key" ON "Retirement"("batchId");

-- CreateIndex
CREATE UNIQUE INDEX "Retirement_certificateUrl_key" ON "Retirement"("certificateUrl");

-- CreateIndex
CREATE INDEX "MonitoringRecord_projectOnChainId_idx" ON "MonitoringRecord"("projectOnChainId");

-- CreateIndex
CREATE UNIQUE INDEX "BenchmarkPrice_methodology_vintageYear_key" ON "BenchmarkPrice"("methodology", "vintageYear");

-- AddForeignKey
ALTER TABLE "Project" ADD CONSTRAINT "Project_developerId_fkey" FOREIGN KEY ("developerId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "CreditBatch" ADD CONSTRAINT "CreditBatch_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES "Project"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Listing" ADD CONSTRAINT "Listing_batchId_fkey" FOREIGN KEY ("batchId") REFERENCES "CreditBatch"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Listing" ADD CONSTRAINT "Listing_sellerId_fkey" FOREIGN KEY ("sellerId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Retirement" ADD CONSTRAINT "Retirement_batchId_fkey" FOREIGN KEY ("batchId") REFERENCES "CreditBatch"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Retirement" ADD CONSTRAINT "Retirement_retiredById_fkey" FOREIGN KEY ("retiredById") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
