```csharp title="C#"
using Kreuzberg;

// Render a single page using the iterator
using var iter = PdfPageIterator.Open("document.pdf", dpi: 150);
foreach (var page in iter)
{
    File.WriteAllBytes("first_page.png", page.Data);
    break; // Only need the first page
}
```
