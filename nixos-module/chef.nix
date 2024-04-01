{ pkgs, config, lib, ... }:
let
  cfg = config.dresden-zone.chef;
in
{
  options.dresden-zone.chef = with lib; {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = ''Wether to enable chef'';
    };
    host = mkOption {
      type = types.str;
      default = "127.0.0.1";
      description = ''
        ip address of chef
      '';
    };
    port = mkOption {
      type = types.port;
      default = 8080;
      description = ''
        port address of chef
      '';
    };
    database = {
      host = mkOption {
        type = types.str;
        default = "127.0.0.1";
        description = ''
          Database host
        '';
      };
      port = mkOption {
        type = types.port;
        default = 5354;
        description = ''
          Database port
        '';
      };
      user = mkOption {
        type = types.str;
        default = "chef";
        description = ''
          user for postgres
        '';
      };
      database = mkOption {
        type = types.str;
        default = "tlms";
        description = ''
          postgres database to use
        '';
      };
      passwordFile = mkOption {
        type = types.either types.path types.string;
        default = "";
        description = ''password file from which the postgres password can be read'';
      };
    };
    user = mkOption {
      type = types.str;
      default = "chef";
      description = ''systemd user'';
    };
    group = mkOption {
      type = types.str;
      default = "chef";
      description = ''group of systemd user'';
    };
    log_level = mkOption {
      type = types.str;
      default = "info";
      description = ''log level of the application'';
    };
  };

  config = lib.mkIf cfg.enable {
    systemd = {
      services = {
        "chef" = {
          enable = true;
          wantedBy = [ "multi-user.target" "network.target" ];

          script = ''
            exec ${pkgs.chef}/bin/chef --listend-addr ${cfg.host}:${toString cfg.port}&
          '';

          environment = {
            "RUST_LOG" = "${cfg.log_level}";
            "RUST_BACKTRACE" = if (cfg.log_level == "info") then "0" else "1";
            "POSTGRES_HOST" = "${cfg.database.host}";
            "POSTGRES_PORT" = "${toString cfg.database.port}";
            "POSTGRES_USER" = "${toString cfg.database.user}";
            "POSTGRES_DATABASE" = "${toString cfg.database.database}";
            "POSTGRES_PASSWORD_PATH" = "${cfg.database.passwordFile}";
          };

          serviceConfig = {
            Type = "forking";
            User = cfg.user;
            Restart = "always";
          };
        };
      };
    };

    users.groups."${cfg.group}" = {};

    # user accounts for systemd units
    users.users."${cfg.user}" = {
      name = "${cfg.user}";
      description = "runs chef";
      isNormalUser = false;
      isSystemUser = true;
      group = cfg.group;
      uid = 1601;
    };
  };
}
