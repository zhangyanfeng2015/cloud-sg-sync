export interface SgPermissionRow {
  description: string;
  portRange: string;
  ipProtocol: string;
  sourceCidrIp?: string | null;
  destCidrIp?: string | null;
  direction: string;
  policy: string;
  nicType: string;
}

export interface SecurityGroupDetail {
  regionId: string;
  securityGroupId: string;
  securityGroupName: string;
  vpcId?: string | null;
  innerDescription?: string | null;
  permissions: SgPermissionRow[];
}
