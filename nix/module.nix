{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.beerio;
in
{
  options.services.beerio = {
    enable = lib.mkEnableOption "beerio server";

    package = lib.mkOption {
      type = lib.types.package;
      description = "The beerio package to use";
    };

    port = lib.mkOption {
      type = lib.types.int;
      default = 3000;
      description = "Port to listen on";
    };
  };

  config = lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts = [ cfg.port ];

    systemd.services.beerio = {
      description = "Beerio Rust Server";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      environment = {
        PORT = toString cfg.port;
      };
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/beerio";
        Restart = "always";
        User = "marty"; # Running as user marty, or create a dedicated user
      };
    };
  };
}
