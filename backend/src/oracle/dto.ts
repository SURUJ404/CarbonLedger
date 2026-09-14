import { IsInt, IsNumberString, IsString } from 'class-validator';

export class SubmitMonitoringDto {
  @IsString() projectOnChainId: string;
  @IsString() dataHash: string;
}

export class UpdatePriceDto {
  @IsString() methodology: string;
  @IsInt() vintageYear: number;
  @IsNumberString() pricePerTonne: string;
}
