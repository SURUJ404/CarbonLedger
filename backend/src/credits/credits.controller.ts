import { Body, Controller, Get, Param, Post, Req, UseGuards } from '@nestjs/common';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { CreditsService } from './credits.service';
import { RecordMintDto, RetireCreditsDto } from './dto';

@Controller('credits')
export class CreditsController {
  constructor(private credits: CreditsService) {}

  @Get(':onChainId')
  getBatch(@Param('onChainId') onChainId: string) {
    return this.credits.getBatch(onChainId);
  }

  @Get('serial/:serial')
  lookupBySerial(@Param('serial') serial: string) {
    return this.credits.lookupBySerial(serial);
  }

  @UseGuards(JwtAuthGuard)
  @Post()
  recordMint(@Body() dto: RecordMintDto) {
    return this.credits.recordMint(dto);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':onChainId/retire')
  retire(
    @Req() req: any,
    @Param('onChainId') onChainId: string,
    @Body() dto: RetireCreditsDto,
  ) {
    return this.credits.recordRetirement(onChainId, req.user.userId, dto);
  }
}
