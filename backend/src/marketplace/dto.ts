import { IsString } from 'class-validator';

export class RecordListingDto {
  @IsString() onChainListingId: string;
  @IsString() onChainBatchId: string;
  @IsString() pricePerTonne: string; // USDC stroops, as string to avoid precision loss
  @IsString() tonnes: string;
}
