import { IsInt, IsString, Min } from 'class-validator';

export class RegisterProjectDto {
  @IsString() name: string;
  @IsString() methodology: string;
  @IsInt() latMicro: number;
  @IsInt() lngMicro: number;
  @IsString() onChainId: string; // returned by contract after register_project()
}

export class VerifyProjectDto {
  @IsInt() @Min(0) methodologyScore: number;
}
