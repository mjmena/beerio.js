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

    domain = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Domain name to serve on (enables Nginx)";
    };

    useACME = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Whether to use ACME for automatic SSL";
    };
  };

  config = lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts = [ cfg.port ];

    services.nginx = lib.mkIf (cfg.domain != null) {
      enable = true;
      virtualHosts.${cfg.domain} = {
        enableACME = cfg.useACME;
        forceSSL = cfg.useACME;
        locations."/" = {
          proxyPass = "http://localhost:${toString cfg.port}";
        };
      };
    };

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
