# LSP semantic intelligence test matrix

| Area | Case | Unit | RPC | VS Code | Negative |
|---|---|---:|---:|---:|---:|
| literals | Int/String/Bool | yes | inlay | smoke | |
| bindings | alias/shadow/reassign | yes | | | yes |
| flow | branch join / union | yes | | | |
| unions | dedupe + widening boundary | yes | | | yes |
| callables | return summaries | yes | completion | smoke | |
| callables | recursion converges | yes | | | yes |
| parameters | call-site inference | yes | optional | | |
| fields | constructor field write | yes | completion | | |
| constructor | constructor vs arbitrary class method | yes | completion | | yes |
| members | declared method/getter | yes | completion | smoke | |
| members | inherited surface | yes | completion | smoke | |
| members | override de-duplication | yes | completion | | yes |
| sides | class-side vs instance-side | yes | completion | | yes |
| self | explicit self | yes | completion | | |
| super | lexical superclass lookup | yes | completion/definition | | yes |
| chains | return-expression receiver | yes | completion | | |
| unknown | no fabricated type | yes | inlay/completion | | yes |
| modules | imported module members | yes | completion | | |
| identity | A.User != B.User | yes | completion | | yes |
| invalidation | same-file didChange | yes | completion | VS Code | yes |
| invalidation | imported provider didChange | yes | completion | | yes |
| hints | stable facts | yes | inlay | smoke | |
| hints | Unknown hidden | yes | inlay | | yes |
| consistency | hint/hover/completion agree | | RPC | | yes |
| syntax | current lexer/parser surface | | semantic tokens | | |
| recovery | trailing dot/incomplete class | | completion | | |
| core | live core source members | yes | completion | | |
| native | no contract => Unknown | yes | inlay | | yes |
| concurrency | coherent revision snapshot | stress | RPC | | yes |
| performance | query/edit targets | bench | optional | | |
