```csharp title="C#"
using Kreuzberg;

// Iterate all pages (memory-efficient, one page at a time)
using var iter = PdfPageIterator.Open("document.pdf", dpi: 150);
foreach (var page in iter)
{
    Console.WriteLine($"Page {page.PageIndex}: {page.Data.Length} bytes");
    File.WriteAllBytes($"page_{page.PageIndex}.png", page.Data);
}
```
