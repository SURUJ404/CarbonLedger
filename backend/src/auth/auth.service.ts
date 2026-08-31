import { Injectable, UnauthorizedException } from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { Keypair } from '@stellar/stellar-sdk';
import { PrismaService } from '../prisma/prisma.service';

// Challenge-response wallet auth, in the spirit of SEP-0030: the server
// hands out a random nonce, the wallet signs it with the account's Stellar
// keypair, and we verify the signature before minting a JWT. No passwords,
// no custody of keys.
const challenges = new Map<string, string>();

@Injectable()
export class AuthService {
  constructor(private prisma: PrismaService, private jwt: JwtService) {}

  issueChallenge(stellarPubKey: string): { nonce: string } {
    const nonce = `CarbonLedger auth: ${Date.now()}-${Math.random().toString(36).slice(2)}`;
    challenges.set(stellarPubKey, nonce);
    return { nonce };
  }

  async verifyAndLogin(stellarPubKey: string, signatureBase64: string) {
    const nonce = challenges.get(stellarPubKey);
    if (!nonce) {
      throw new UnauthorizedException('No challenge issued for this account');
    }

    const keypair = Keypair.fromPublicKey(stellarPubKey);
    const verified = keypair.verify(
      Buffer.from(nonce, 'utf-8'),
      Buffer.from(signatureBase64, 'base64'),
    );
    if (!verified) {
      throw new UnauthorizedException('Invalid signature');
    }
    challenges.delete(stellarPubKey);

    const user = await this.prisma.user.upsert({
      where: { stellarPubKey },
      update: {},
      create: { stellarPubKey, role: 'DEVELOPER' },
    });

    const token = this.jwt.sign({ sub: user.id, pub: user.stellarPubKey, role: user.role });
    return { accessToken: token, user };
  }
}
