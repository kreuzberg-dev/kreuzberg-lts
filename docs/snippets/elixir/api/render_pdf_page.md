```elixir title="Elixir"
# Render a single page (zero-based index)
{:ok, png} = Kreuzberg.render_pdf_page("document.pdf", 0, dpi: 150)

File.write!("first_page.png", png)
```
