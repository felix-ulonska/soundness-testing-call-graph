{pkgs}: {
  awk = pkgs.busybox.override {
    #enableMinimal = true;
    enableAppletSymlinks = false;
    enableStatic = false; # optional, comment this out if not needed
    extraConfig = ''
      CONFIG_AWK=y
    '';
  };
}
