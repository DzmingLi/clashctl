{ config, lib, ... }:

let
  cfg = config.programs.clashctl;

  inherit (lib)
    mkEnableOption mkOption mkIf mkMerge types
    concatStringsSep mapAttrsToList
    literalExpression;

  # Convert a single keybinding attrset to RON
  keyBindingToRON = kb:
    let
      modsStr =
        if kb.modifiers == [] then "modifiers: []"
        else "modifiers: [${concatStringsSep ", " (map (m: "\"${m}\"") kb.modifiers)}]";
    in
      "(key: \"${kb.key}\", ${modsStr})";

  # Convert a list of keybindings to RON array
  keyBindingsListToRON = bindings:
    "[${concatStringsSep ", " (map keyBindingToRON bindings)}]";

  # Convert the full keybindings config to RON
  keybindingsToRON = kb:
    let
      entries = mapAttrsToList (name: bindings:
        "${name}: ${keyBindingsListToRON bindings},"
      ) kb;
    in
      concatStringsSep "\n        " entries;

  # Convert a server to RON
  serverToRON = srv:
    let
      secretPart = if srv.secret != null
        then "secret: Some(\"${srv.secret}\")"
        else "secret: None";
    in
      "(url: \"${srv.url}\", ${secretPart})";

  # Generate subscription RON section
  subscriptionRON =
    if cfg.subscription.enable then
      let
        urlPart =
          if cfg.subscription.url != null
          then "url: Some(\"${cfg.subscription.url}\")"
          else "url: None";
        urlFilePart =
          if cfg.subscription.urlFile != null
          then "url_file: Some(\"${toString cfg.subscription.urlFile}\")"
          else "url_file: None";
        uaPart =
          if cfg.subscription.userAgent != null
          then "user_agent: Some(\"${cfg.subscription.userAgent}\")"
          else "user_agent: None";
        overridePart =
          if cfg.subscription.overrides != {}
          then "override_file: Some(\"${config.home.homeDirectory}/.config/clashctl/overrides.yaml\")"
          else "override_file: None";
      in
        "Some((${urlPart}, ${urlFilePart}, ${uaPart}, ${overridePart}))"
    else
      "None";

  # Build the full config.ron
  configRON = ''
    (
        servers: [${concatStringsSep ", " (map serverToRON cfg.servers)}],
        using: ${if cfg.using != null then "Some(\"${cfg.using}\")" else "None"},
        tui: (
            log_file: None,
            subscription: ${subscriptionRON},
        ),
        sort: (
            connections: Name,
            rules: Name,
            proxies: Name,
        ),
        keybindings: (
            ${keybindingsToRON mergedKeybindings}
        ),
    )
  '';

  # Default keybindings matching Rust defaults
  defaultKeybindings = {
    quit = [{ key = "q"; modifiers = []; } { key = "x"; modifiers = []; } { key = "c"; modifiers = ["ctrl"]; }];
    test_latency = [{ key = "t"; modifiers = []; }];
    toggle_hold = [{ key = "space"; modifiers = []; }];
    toggle_debug = [{ key = "d"; modifiers = ["ctrl"]; }];
    next_sort = [{ key = "s"; modifiers = []; }];
    prev_sort = [{ key = "s"; modifiers = ["alt"]; }];
    refresh_subscription = [{ key = "r"; modifiers = []; }];
    tab_goto = builtins.genList (i: { key = toString (i + 1); modifiers = []; }) 9;
  };

  # Merge user overrides with defaults
  mergedKeybindings = defaultKeybindings // cfg.keybindings;

  keyBindingSubmodule = types.submodule {
    options = {
      key = mkOption {
        type = types.str;
        description = "Key name (e.g. \"q\", \"space\", \"F5\", \"esc\")";
      };
      modifiers = mkOption {
        type = types.listOf (types.enum [ "ctrl" "alt" "shift" ]);
        default = [];
        description = "Key modifiers";
      };
    };
  };

  serverSubmodule = types.submodule {
    options = {
      url = mkOption {
        type = types.str;
        default = "http://127.0.0.1:9090";
        description = "Clash RESTful API URL";
      };
      secret = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Bearer token secret";
      };
    };
  };

in {
  options.programs.clashctl = {
    enable = mkEnableOption "clashctl - Clash TUI controller";

    package = mkOption {
      type = types.package;
      description = "The clashctl package to use";
    };

    servers = mkOption {
      type = types.listOf serverSubmodule;
      default = [{ url = "http://127.0.0.1:9090"; }];
      description = "List of Clash API servers";
    };

    using = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "URL of the active server";
    };

    keybindings = mkOption {
      type = types.attrsOf (types.listOf keyBindingSubmodule);
      default = {};
      description = "Custom keybinding overrides. Keys: quit, test_latency, toggle_hold, toggle_debug, next_sort, prev_sort, refresh_subscription, tab_goto";
      example = literalExpression ''
        {
          quit = [{ key = "q"; } { key = "c"; modifiers = ["ctrl"]; }];
          refresh_subscription = [{ key = "r"; } { key = "F5"; }];
        }
      '';
    };

    subscription = {
      enable = mkEnableOption "subscription link management";

      url = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Subscription URL (plaintext, not recommended for secrets)";
      };

      urlFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to file containing the subscription URL (e.g. /run/agenix/clash-subscription-url). Preferred for agenix integration.";
      };

      userAgent = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Custom User-Agent header for subscription requests";
      };

      overrides = mkOption {
        type = types.attrs;
        default = {};
        description = ''
          Override subscription config on refresh.
          - prepend-rules: list prepended to rules (higher priority)
          - append-rules: list appended to rules
          - Other keys: deep merged (override wins)
        '';
        example = literalExpression ''
          {
            mixed-port = 7890;
            prepend-rules = [
              "DOMAIN-SUFFIX,openai.com,PROXY"
            ];
          }
        '';
      };

      refreshInterval = mkOption {
        type = types.str;
        default = "*-*-* 0/12:00:00";
        description = "systemd timer OnCalendar value for automatic refresh";
        example = "hourly";
      };
    };
  };

  config = mkIf cfg.enable (mkMerge [
    {
      home.packages = [ cfg.package ];

      # Generate ~/.config/clashctl/config.ron
      xdg.configFile."clashctl/config.ron".text = configRON;

      # Generate overrides YAML if overrides are set
      xdg.configFile."clashctl/overrides.yaml" = mkIf (cfg.subscription.overrides != {}) {
        text = builtins.toJSON cfg.subscription.overrides;
      };
    }

    # Subscription auto-refresh via systemd user timer
    (mkIf cfg.subscription.enable {
      assertions = [
        {
          assertion = cfg.subscription.url != null || cfg.subscription.urlFile != null;
          message = "programs.clashctl.subscription: either url or urlFile must be set";
        }
      ];

      systemd.user.services.clashctl-refresh = {
        Unit.Description = "Refresh Clash subscription";
        Service = {
          Type = "oneshot";
          ExecStart = "${cfg.package}/bin/clashctl subscription refresh";
        };
      };

      systemd.user.timers.clashctl-refresh = {
        Unit.Description = "Timer for Clash subscription refresh";
        Install.WantedBy = [ "timers.target" ];
        Timer = {
          OnCalendar = cfg.subscription.refreshInterval;
          Persistent = true;
        };
      };
    })
  ]);
}
