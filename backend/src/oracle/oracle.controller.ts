import { Body, Controller, Get, Headers, Param, Post, UnauthorizedException } from '@nestjs/common';
import { OracleService } from './oracle.service';
import { SubmitMonitoringDto, UpdatePriceDto } from './dto';

// These endpoints are called by the Python oracle bridge (see /oracle),
// authenticated with a shared secret rather than user JWTs since no wallet
// is involved.
function checkOracleSecret(header?: string) {
  if (header !== process.env.ORACLE_SHARED_SECRET) {
    throw new UnauthorizedException('Invalid oracle credentials');
  }
}

@Controller('oracle')
export class OracleController {
  constructor(private oracle: OracleService) {}

  @Post('monitoring')
  submitMonitoring(
    @Headers('x-oracle-secret') secret: string,
    @Body() dto: SubmitMonitoringDto,
  ) {
    checkOracleSecret(secret);
    return this.oracle.recordMonitoring(dto);
  }

  @Get('monitoring/:projectOnChainId/current')
  isCurrent(@Param('projectOnChainId') id: string) {
    return this.oracle.isMonitoringCurrent(id);
  }

  @Post('price')
  updatePrice(@Headers('x-oracle-secret') secret: string, @Body() dto: UpdatePriceDto) {
    checkOracleSecret(secret);
    return this.oracle.updatePrice(dto);
  }

  @Get('price/:methodology/:vintageYear')
  getPrice(@Param('methodology') methodology: string, @Param('vintageYear') vintageYear: string) {
    return this.oracle.getPrice(methodology, parseInt(vintageYear, 10));
  }
}
