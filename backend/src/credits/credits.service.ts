import { Injectable, NotFoundException } from '@nestjs/common';
import { randomUUID } from 'crypto';
import { PrismaService } from '../prisma/prisma.service';
import { RecordMintDto, RetireCreditsDto } from './dto';

@Injectable()
export class CreditsService {
  constructor(private prisma: PrismaService) {}

  async recordMint(dto: RecordMintDto) {
    const project = await this.prisma.project.findUnique({
      where: { onChainId: BigInt(dto.projectOnChainId) },
    });
    if (!project) throw new NotFoundException('Project not found');

    return this.prisma.creditBatch.create({
      data: {
        onChainId: BigInt(dto.onChainBatchId),
        projectId: project.id,
        vintageYear: dto.vintageYear,
        serialStart: BigInt(dto.serialStart),
        serialEnd: BigInt(dto.serialEnd),
        ownerPubKey: dto.ownerPubKey,
      },
    });
  }

  async getBatch(onChainId: string) {
    const batch = await this.prisma.creditBatch.findUnique({
      where: { onChainId: BigInt(onChainId) },
      include: { project: true, retirement: true },
    });
    if (!batch) throw new NotFoundException('Credit batch not found');
    return batch;
  }

  /// Records a confirmed on-chain retire_credits() call and issues a
  /// permanent, publicly-verifiable certificate URL - the off-chain
  /// counterpart to the contract's get_retirement_certificate().
  async recordRetirement(
    onChainBatchId: string,
    retiredById: string,
    dto: RetireCreditsDto,
  ) {
    const batch = await this.prisma.creditBatch.update({
      where: { onChainId: BigInt(onChainBatchId) },
      data: { retired: true },
    });

    const tonnes = batch.serialEnd - batch.serialStart + BigInt(1);
    const certificateSlug = randomUUID();

    return this.prisma.retirement.create({
      data: {
        batchId: batch.id,
        retiredById,
        beneficiary: dto.beneficiary,
        reason: dto.reason,
        tonnes,
        certificateUrl: `${process.env.PUBLIC_BASE_URL || 'https://carbonledger.io'}/certificates/${certificateSlug}`,
      },
    });
  }

  async lookupBySerial(serial: string) {
    const s = BigInt(serial);
    const batch = await this.prisma.creditBatch.findFirst({
      where: { serialStart: { lte: s }, serialEnd: { gte: s } },
      include: { project: true, retirement: true },
    });
    if (!batch) throw new NotFoundException('No batch contains that serial number');
    return batch;
  }
}
