import { Module } from '@nestjs/common';
import { ProjectsService } from './projects.service';
import { ProjectsController } from './projects.controller';
import { SorobanService } from '../stellar/soroban.service';

@Module({
  providers: [ProjectsService, SorobanService],
  controllers: [ProjectsController],
  exports: [ProjectsService],
})
export class ProjectsModule {}
