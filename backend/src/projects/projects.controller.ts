import { Body, Controller, Get, Param, Post, Query, Req, UseGuards } from '@nestjs/common';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { RolesGuard, Roles } from '../auth/roles.guard';
import { ProjectsService } from './projects.service';
import { RegisterProjectDto, VerifyProjectDto } from './dto';

@Controller('projects')
export class ProjectsController {
  constructor(private projects: ProjectsService) {}

  @Get()
  findAll(@Query('status') status?: 'PENDING' | 'VERIFIED' | 'SUSPENDED' | 'REJECTED') {
    return this.projects.findAll(status);
  }

  @Get(':onChainId')
  findOne(@Param('onChainId') onChainId: string) {
    return this.projects.findOne(onChainId);
  }

  // Called by the frontend after the wallet signs & submits register_project()
  // on-chain, to index the result for fast querying.
  @UseGuards(JwtAuthGuard)
  @Post()
  register(@Req() req: any, @Body() dto: RegisterProjectDto) {
    return this.projects.recordRegistration(req.user.userId, dto);
  }

  // Called after verify_project() confirms on-chain. Verifier-role only.
  @UseGuards(JwtAuthGuard, RolesGuard)
  @Roles('VERIFIER', 'ADMIN')
  @Post(':onChainId/verify')
  verify(@Param('onChainId') onChainId: string, @Body() dto: VerifyProjectDto) {
    return this.projects.markVerified(onChainId, dto.methodologyScore);
  }
}
