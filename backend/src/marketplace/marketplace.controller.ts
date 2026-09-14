import { Body, Controller, Get, Param, Post, Query, Req, UseGuards } from '@nestjs/common';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { MarketplaceService } from './marketplace.service';
import { RecordListingDto } from './dto';

@Controller('marketplace')
export class MarketplaceController {
  constructor(private marketplace: MarketplaceService) {}

  @Get('listings')
  getActiveListings(
    @Query('methodology') methodology?: string,
    @Query('vintageYear') vintageYear?: string,
  ) {
    return this.marketplace.getActiveListings(
      methodology,
      vintageYear ? parseInt(vintageYear, 10) : undefined,
    );
  }

  @UseGuards(JwtAuthGuard)
  @Post('listings')
  recordListing(@Req() req: any, @Body() dto: RecordListingDto) {
    return this.marketplace.recordListing(req.user.userId, dto);
  }

  @UseGuards(JwtAuthGuard)
  @Post('listings/:onChainListingId/delist')
  delist(@Param('onChainListingId') id: string) {
    return this.marketplace.recordDelist(id);
  }

  @UseGuards(JwtAuthGuard)
  @Post('listings/:onChainListingId/purchase')
  purchase(@Req() req: any, @Param('onChainListingId') id: string) {
    return this.marketplace.recordPurchase(id, req.user.stellarPubKey);
  }
}
