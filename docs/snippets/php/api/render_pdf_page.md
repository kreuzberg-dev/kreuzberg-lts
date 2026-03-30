```php title="PHP"
<?php

declare(strict_types=1);

use function Kreuzberg\render_pdf_page;

// Render a single page (zero-based index)
$png = render_pdf_page('document.pdf', 0, 150);

file_put_contents('first_page.png', $png);
```
