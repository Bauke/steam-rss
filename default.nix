{ lib, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "steam-rss";
  version = "0.2.2";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  meta = with lib; {
    description = "Get RSS feeds for Steam games";
    homepage = "https://github.com/Bauke/steam-rss";
    changelog = "https://github.com/Bauke/steam-rss/releases/tag/${version}";
    license = with licenses; [ agpl3Plus ];
    maintainers = with maintainers; [ Bauke ];
    mainProgram = "steam-rss";
  };
}
