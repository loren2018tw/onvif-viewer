export interface CameraInfo {
  id: string;
  name: string;
  address: string;
  onvif_port: number;
  username: string;
  password: string;
  stream_uri: string | null;
  manufacturer: string | null;
  model: string | null;
  created_at: string;
  updated_at: string;
}

export interface DiscoveredCamera {
  address: string;
  port: number;
  name: string | null;
  manufacturer: string | null;
  model: string | null;
  xaddrs: string[];
}

export interface ScanRange {
  start_ip: string;
  end_ip: string;
  port: number;
}

export interface FFmpegStatus {
  installed: boolean;
  install_command: string;
}

export type StreamType = "main" | "sub";
