%{
configs: [
%{
name: "default",
files: %{
included: [
"lib/",
"src/",
"test/",
"web/",
"apps/*/lib/",
"apps/*/src/",
"apps/*/test/",
"apps/*/web/"
],
excluded: [~r"/_build/", ~r"/deps/", ~r"/node_modules/"]
},
plugins: [],
requires: [],
strict: true,
parse_timeout: 5000,
color: true,
checks: %{
extra: [],
enabled: [
{Credo.Check.Refactor.CyclomaticComplexity, max_complexity: 16}
]
}
}
]
}
