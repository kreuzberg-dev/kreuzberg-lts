```ruby title="Ruby"
require 'kreuzberg'

# Render a single page (zero-based index)
png = Kreuzberg.render_pdf_page('document.pdf', 0, dpi: 150)

File.binwrite('first_page.png', png)
```
