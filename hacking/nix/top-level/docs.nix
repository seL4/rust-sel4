{ lib, pkgs }:

let
  metaCrateName = "meta_for_docs";

  worlds = [
    (
      let
        pkgSet = pkgs.host.aarch64.none;
        world = pkgSet.this.worlds.default;
      in rec {
        name = pkgSet.this.defaultRustTargetName;
        description = "${name} (qemu-virt-arm)";
        byRuntime = world.docs;
      }
    )
    (
      let
        pkgSet = pkgs.host.riscv64.none;
        world = pkgSet.this.worlds.default;
      in rec {
        name = pkgSet.this.defaultRustTargetName;
        description = "${name} (spike)";
        byRuntime = world.docs;
      }
    )
    (
      let
        pkgSet = pkgs.host.x86_64.none;
        world = pkgSet.this.worlds.default;
      in rec {
        name = pkgSet.this.defaultRustTargetName;
        description = "${name} (pc99)";
        byRuntime = world.docs;
      }
    )
  ];

  mk = { worlds }: rec {

    html = rustdocHtml;

    # html = pkgs.build.linkFarm "top-level-html" [
    #   { name = "rustdoc"; path = rustdocHtml; }
    # ];

    rustdocHtml = pkgs.build.linkFarm "rustdoc-html" ([
      { name = "index.html";
        path = rustdocIndex;
      }
    ] ++
      lib.concatLists
        (lib.forEach worlds (world:
          lib.forEach world.byRuntime (runtime: {
            name = "worlds/${world.name}/runtimes/${runtime.name}";
            path = runtime.drv;
          })))
    );

    rustdocIndex = pkgs.build.writeText "index.html" ''
      <!DOCTYPE html>
      <html>
        <head>
          <meta charset="utf-8">
          <meta name="viewport" content="width=device-width, initial-scale=1">
          <title>Rustdoc for rust-seL4</title>
          <link
            rel="stylesheet"
            href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.0.0/github-markdown-light.min.css"
            integrity="sha512-2ZxkJRe/dlKUknBZJNP93bh08JvvuvL+fR6I3IqZ4tnAvNQ0D56+LVg+DvE/S/Ir4J/6lxBu/Xye1z243BEa1Q=="
            crossorigin="anonymous"
            referrerpolicy="no-referrer"
          />
          <style>
            .markdown-body {
              box-sizing: border-box;
              min-width: 200px;
              max-width: 980px;
              margin: 0 auto;
              padding: 45px;
            }
            @media (max-width: 767px) {
              .markdown-body {
                padding: 15px;
              }
            }
          </style>
        </head>
        <body>
          <div class="markdown-body">
            <h1>Rustdoc for rust-seL4</h1>
            <p>
              <ul>
                ${lib.concatStrings
                  (lib.forEach worlds (world: ''
                    <li>
                      ${world.description}, with runtime:
                        <ul>
                          ${lib.concatStrings
                            (lib.forEach world.byRuntime (runtime: ''
                              <li>
                                <a href="./worlds/${world.name}/runtimes/${runtime.name}/${world.name}/doc/${metaCrateName}/index.html">${runtime.name}</a>
                              </li>
                            ''))
                          }
                        </ul>
                    </li>
                  ''))
                }
              </ul>
            </p>
          </div>
        </body>
      </html>
    '';
  };

in rec {

  realized = mk { inherit worlds; };

  inherit (realized) html;

  htmlCopied = pkgs.build.runCommand "html" {} ''
    cp -rL ${realized.html} $out
  '';

}
