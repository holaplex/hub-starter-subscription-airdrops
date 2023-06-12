-- CreateTable
CREATE TABLE "Airdrop" (
    "dropId" TEXT NOT NULL,
    "startedAt" TIMESTAMP(3),
    "completedAt" TIMESTAMP(3)
);

-- CreateIndex
CREATE UNIQUE INDEX "Airdrop_dropId_key" ON "Airdrop"("dropId");
