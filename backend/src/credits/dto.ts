import { IsInt, IsString, Min } from 'class-validator';

export class RecordMintDto {
  @IsString() onChainBatchId: string;
  @IsString() projectOnChainId: string;
  @IsInt() vintageYear: number;
  @IsString() serialStart: string;
  @IsString() serialEnd: string;
  @IsString() ownerPubKey: string;
}

export class RetireCreditsDto {
  @IsString() beneficiary: string;
  @IsString() reason: string;
}
