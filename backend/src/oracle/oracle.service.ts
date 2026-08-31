import { Injectable } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { SubmitMonitoringDto, UpdatePriceDto } from './dto';

const FRESHNESS_DAYS = 365;

@Injectable()
export class OracleService {
  constructor(private prisma: PrismaService) {}

  async recordMonitoring(dto: SubmitMonitoringDto) {
    const record = await this.prisma.monitoringRecord.create({
      data: {
        projectOnChainId: BigInt(dto.projectOnChainId),
        dataHash: dto.dataHash,
      },
    });
    await this.prisma.project.update({
      where: { onChainId: BigInt(dto.projectOnChainId) },
      data: { lastMonitoringAt: new Date() },
    });
    return record;
  }

  async isMonitoringCurrent(projectOnChainId: string) {
    const project = await this.prisma.project.findUnique({
      where: { onChainId: BigInt(projectOnChainId) },
    });
    if (!project?.lastMonitoringAt) return { current: false };
    const ageDays =
      (Date.now() - project.lastMonitoringAt.getTime()) / (1000 * 60 * 60 * 24);
    return { current: ageDays <= FRESHNESS_DAYS, ageDays: Math.floor(ageDays) };
  }

  async updatePrice(dto: UpdatePriceDto) {
    return this.prisma.benchmarkPrice.upsert({
      where: {
        methodology_vintageYear: {
          methodology: dto.methodology,
          vintageYear: dto.vintageYear,
        },
      },
      update: { pricePerTonne: BigInt(dto.pricePerTonne) },
      create: {
        methodology: dto.methodology,
        vintageYear: dto.vintageYear,
        pricePerTonne: BigInt(dto.pricePerTonne),
      },
    });
  }

  async getPrice(methodology: string, vintageYear: number) {
    return this.prisma.benchmarkPrice.findUnique({
      where: { methodology_vintageYear: { methodology, vintageYear } },
    });
  }
}
