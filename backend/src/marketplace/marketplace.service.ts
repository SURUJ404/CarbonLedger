import { Injectable, NotFoundException } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { RecordListingDto } from './dto';

// NOTE ON SCOPE: the original marketplace supported bulk_purchase() across
// multiple projects in one transaction and secondary trading via the
// Stellar DEX (SDEX). Both are intentionally dropped here - this service
// only tracks single listings and single purchases, mirroring the
// simplified carbon_marketplace contract.
@Injectable()
export class MarketplaceService {
  constructor(private prisma: PrismaService) {}

  async recordListing(sellerId: string, dto: RecordListingDto) {
    const batch = await this.prisma.creditBatch.findUnique({
      where: { onChainId: BigInt(dto.onChainBatchId) },
    });
    if (!batch) throw new NotFoundException('Credit batch not found');

    return this.prisma.listing.create({
      data: {
        onChainId: BigInt(dto.onChainListingId),
        batchId: batch.id,
        sellerId,
        pricePerTonne: BigInt(dto.pricePerTonne),
        tonnes: BigInt(dto.tonnes),
      },
    });
  }

  async recordDelist(onChainListingId: string) {
    return this.prisma.listing.update({
      where: { onChainId: BigInt(onChainListingId) },
      data: { active: false },
    });
  }

  async recordPurchase(onChainListingId: string, buyerPubKey: string) {
    const listing = await this.prisma.listing.update({
      where: { onChainId: BigInt(onChainListingId) },
      data: { active: false },
      include: { batch: true },
    });
    await this.prisma.creditBatch.update({
      where: { id: listing.batchId },
      data: { ownerPubKey: buyerPubKey },
    });
    return listing;
  }

  async getActiveListings(methodology?: string, vintageYear?: number) {
    return this.prisma.listing.findMany({
      where: {
        active: true,
        batch: {
          vintageYear: vintageYear ?? undefined,
          project: methodology ? { methodology } : undefined,
        },
      },
      include: { batch: { include: { project: true } }, seller: true },
      orderBy: { createdAt: 'desc' },
    });
  }
}
