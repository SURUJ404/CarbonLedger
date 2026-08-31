import { Injectable, NotFoundException } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { RegisterProjectDto } from './dto';

// Projects live on-chain in carbon_registry; this service maintains a
// PostgreSQL mirror for fast search/filter/pagination, populated after a
// transaction confirms. The chain is always the source of truth for status.
@Injectable()
export class ProjectsService {
  constructor(private prisma: PrismaService) {}

  async recordRegistration(developerId: string, dto: RegisterProjectDto) {
    return this.prisma.project.create({
      data: {
        onChainId: BigInt(dto.onChainId),
        developerId,
        name: dto.name,
        methodology: dto.methodology,
        latMicro: BigInt(dto.latMicro),
        lngMicro: BigInt(dto.lngMicro),
        status: 'PENDING',
      },
    });
  }

  async markVerified(onChainId: string, methodologyScore: number) {
    return this.prisma.project.update({
      where: { onChainId: BigInt(onChainId) },
      data: { status: 'VERIFIED', methodologyScore },
    });
  }

  async markSuspended(onChainId: string) {
    return this.prisma.project.update({
      where: { onChainId: BigInt(onChainId) },
      data: { status: 'SUSPENDED' },
    });
  }

  async findAll(status?: 'PENDING' | 'VERIFIED' | 'SUSPENDED' | 'REJECTED') {
    return this.prisma.project.findMany({
      where: status ? { status } : undefined,
      include: { developer: true },
      orderBy: { createdAt: 'desc' },
    });
  }

  async findOne(onChainId: string) {
    const project = await this.prisma.project.findUnique({
      where: { onChainId: BigInt(onChainId) },
      include: { developer: true, batches: true },
    });
    if (!project) throw new NotFoundException('Project not found');
    return project;
  }
}
