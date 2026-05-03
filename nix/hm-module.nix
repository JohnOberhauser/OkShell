self:
{ config, lib, pkgs, ... }:

let
  cfg = config.programs.okshell;
in
{
  options.programs.okshell = {
    enable = lib.mkEnableOption "OkShell — a customizable shell for Hyprland";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.okshell;
      defaultText = lib.literalExpression "okshell.packages.\${system}.okshell";
      description = "The okshell package to use.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
  };
}
