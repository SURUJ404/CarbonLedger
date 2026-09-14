import { Body, Controller, Post } from '@nestjs/common';
import { IsString } from 'class-validator';
import { AuthService } from './auth.service';

class ChallengeDto {
  @IsString() stellarPubKey: string;
}

class VerifyDto {
  @IsString() stellarPubKey: string;
  @IsString() signature: string;
}

@Controller('auth')
export class AuthController {
  constructor(private auth: AuthService) {}

  @Post('challenge')
  challenge(@Body() dto: ChallengeDto) {
    return this.auth.issueChallenge(dto.stellarPubKey);
  }

  @Post('verify')
  verify(@Body() dto: VerifyDto) {
    return this.auth.verifyAndLogin(dto.stellarPubKey, dto.signature);
  }
}
