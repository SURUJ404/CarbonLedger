import { Module } from '@nestjs/common';
import { OracleService } from './oracle.service';
import { OracleController } from './oracle.controller';

@Module({
  providers: [OracleService],
  controllers: [OracleController],
})
export class OracleModule {}
