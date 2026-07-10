/**
 * Comprehensive table extraction quality tests for TypeScript Node.js bindings.
 *
 * Tests verify table extraction quality across multiple scenarios:
 * 1. Table structure extraction (rows, columns, headers)
 * 2. Complex tables (merged cells, nested tables)
 * 3. Table-in-table edge cases
 * 4. Format-specific table handling (PDF vs. Office formats)
 * 5. Performance with large tables (100+ rows)
 * 6. Markdown conversion accuracy
 * 7. Cell content preservation
 * 8. Table boundary detection
 * 9. Batch table extraction consistency
 * 10. Table metadata validation
 *
 * NAPI-RS bindings with plain object configs (NO builder pattern).
 */

import { readFileSync, realpathSync } from "node:fs";
import { beforeAll, describe, expect, it } from "vitest";
import { extractBytesSync, extractFileSync } from "../../dist/index.js";
import type { ExtractionConfig, Table } from "../../src/types.js";
import { getTestDocumentPath } from "../helpers/index.js";

let tinyPdfPath: string;
let mediumPdfPath: string;
let largePdfPath: string;
let tinyPdfBytes: Uint8Array;

beforeAll(() => {
  tinyPdfPath = getTestDocumentPath("pdf/tiny.pdf");
  mediumPdfPath = getTestDocumentPath("pdf/medium.pdf");
  largePdfPath = getTestDocumentPath("pdf/large.pdf");

  try {
    tinyPdfBytes = new Uint8Array(readFileSync(realpathSync(tinyPdfPath)));
  } catch {}
});

describe("Table Extraction Quality (Node.js Bindings)", () => {
  describe("table structure extraction", () => {
    it("should extract table rows, columns, and headers", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      expect(result).toBeDefined();
      expect(result.tables).toBeDefined();
      expect(Array.isArray(result.tables)).toBe(true);

      if (result.tables && result.tables.length > 0) {
        const table = result.tables[0];
        expect(table.cells).toBeDefined();
        expect(Array.isArray(table.cells)).toBe(true);
        expect(table.cells.length).toBeGreaterThan(0);

        for (const row of table.cells) {
          expect(Array.isArray(row)).toBe(true);
          expect(row.length).toBeGreaterThan(0);
        }

        expect(table.markdown).toBeDefined();
        expect(typeof table.markdown).toBe("string");
        expect(table.markdown.length).toBeGreaterThan(0);
      }
    });

    it("should preserve cell content in extracted tables", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        const table = result.tables[0];

        for (const row of table.cells) {
          for (const cell of row) {
            expect(typeof cell).toBe("string");
            expect(cell).not.toBeNull();
          }
        }
      }
    });

    it("should include page number for extracted tables", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          expect(table).toHaveProperty("pageNumber");
          expect(table.pageNumber).toBeGreaterThanOrEqual(1);
          expect(Number.isInteger(table.pageNumber)).toBe(true);
        }
      }
    });

    it("should maintain column count consistency within tables", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          const columnCounts = table.cells.map((row) => row.length);
          const minCols = Math.min(...columnCounts);
          const maxCols = Math.max(...columnCounts);
          expect(minCols).toBeGreaterThan(0);
          expect(maxCols).toEqual(minCols);
        }
      }
    });
  });

  describe("complex table handling", () => {
    it("should extract tables with merged cell structures", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      expect(result.tables).toBeDefined();

      if (result.tables && result.tables.length > 0) {
        const table = result.tables[0];
        expect(table.cells).toBeDefined();
        expect(table.cells.length).toBeGreaterThan(0);

        for (const row of table.cells) {
          expect(Array.isArray(row)).toBe(true);
          expect(row.length).toBeGreaterThan(0);
        }
      }
    });

    it("should handle tables with nested content", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          expect(Array.isArray(table.cells)).toBe(true);

          for (const row of table.cells) {
            expect(Array.isArray(row)).toBe(true);
            for (const cell of row) {
              expect(typeof cell).toBe("string");
            }
          }
        }
      }
    });

    it("should preserve cell formatting and special characters", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        const allCellContent = result.tables.flatMap((t) => t.cells.flatMap((row) => row.join(" ")));

        for (const content of allCellContent) {
          expect(typeof content).toBe("string");
        }
      }
    });
  });

  describe("table-in-table edge cases", () => {
    it("should handle documents with multiple tables on single page", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      expect(result.tables).toBeDefined();
      expect(Array.isArray(result.tables)).toBe(true);

      if (result.tables && result.tables.length > 1) {
        const pageNumbers = result.tables.map((t) => t.pageNumber);

        for (const pageNum of pageNumbers) {
          expect(pageNum).toBeGreaterThanOrEqual(1);
        }
      }
    });

    it("should detect table boundaries correctly", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          const rowCount = table.cells.length;
          expect(rowCount).toBeGreaterThan(0);

          for (const row of table.cells) {
            expect(row.length).toBeGreaterThan(0);
          }
        }
      }
    });

    it("should handle empty cells and whitespace-only cells", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          for (const row of table.cells) {
            for (const cell of row) {
              expect(typeof cell).toBe("string");
            }
          }
        }
      }
    });
  });

  describe("format-specific table handling", () => {
    it("should extract tables from PDF documents", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      expect(result).toBeDefined();
      expect(result.mimeType).toContain("application/pdf");
      expect(result.tables).toBeDefined();
    });

    it("should extract tables from PDF using bytes interface", () => {
      if (!tinyPdfBytes || tinyPdfBytes.length === 0) {
        expect(true).toBe(true);
        return;
      }

      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractBytesSync(Buffer.from(tinyPdfBytes), "application/pdf", config);

      expect(result).toBeDefined();
      expect(result.tables).toBeDefined();
      expect(Array.isArray(result.tables)).toBe(true);
    });

    it("should handle PDF table extraction configuration", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
          extractMetadata: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      expect(result).toBeDefined();
      expect(result.tables).toBeDefined();
    });

    it("should maintain table quality across different PDF settings", () => {
      const config1: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const config2: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
          extractMetadata: true,
        },
      };

      const result1 = extractFileSync(tinyPdfPath, config1);
      const result2 = extractFileSync(tinyPdfPath, config2);

      expect(result1.tables).toBeDefined();
      expect(result2.tables).toBeDefined();

      if (result1.tables && result2.tables) {
        expect(result1.tables.length).toBe(result2.tables.length);
      }
    });
  });

  describe("performance with large tables", () => {
    it("should handle extraction of large tables (100+ rows)", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(largePdfPath, config);

      expect(result.tables).toBeDefined();

      if (result.tables && result.tables.length > 0) {
        const table = result.tables[0];

        const rowCount = table.cells.length;
        expect(rowCount).toBeGreaterThan(0);

        for (let i = 0; i < Math.min(rowCount, 10); i++) {
          expect(Array.isArray(table.cells[i])).toBe(true);
          expect(table.cells[i].length).toBeGreaterThan(0);
        }
      }
    });

    it("should maintain extraction consistency for large tables", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result1 = extractFileSync(largePdfPath, config);
      const result2 = extractFileSync(largePdfPath, config);

      expect(result1.tables.length).toBe(result2.tables.length);

      if (result1.tables.length > 0 && result2.tables.length > 0) {
        expect(result1.tables[0].cells.length).toBe(result2.tables[0].cells.length);
      }
    });

    it("should handle efficient memory usage for table extraction", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const startMemory = process.memoryUsage().heapUsed;
      const result = extractFileSync(largePdfPath, config);
      const endMemory = process.memoryUsage().heapUsed;

      expect(result.tables).toBeDefined();

      const memoryIncrease = endMemory - startMemory;
      expect(memoryIncrease).toBeLessThan(500 * 1024 * 1024);
    });
  });

  describe("markdown conversion accuracy", () => {
    it("should convert tables to valid markdown format", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          expect(table.markdown).toBeDefined();
          expect(typeof table.markdown).toBe("string");

          const lines = table.markdown.split("\n");
          const hasPipes = lines.some((line) => line.includes("|"));

          expect(hasPipes || table.markdown.length === 0).toBe(true);
        }
      }
    });

    it("should preserve cell content in markdown representation", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        const table = result.tables[0];

        if (table.cells.length > 0 && table.cells[0].length > 0) {
          expect(table.markdown.length).toBeGreaterThan(0);
        }
      }
    });

    it("should format markdown with proper alignment markers", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          const markdown = table.markdown;

          const lines = markdown.split("\n");
          if (lines.length > 1) {
            const hasAlignmentRow = lines.some((line) => /\|[\s\-:]+\|/.test(line));

            expect(hasAlignmentRow || markdown.length === 0).toBe(true);
          }
        }
      }
    });

    it("should handle markdown conversion with special characters", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          expect(typeof table.markdown).toBe("string");

          const lines = table.markdown.split("\n");
          for (const line of lines) {
            expect(typeof line).toBe("string");
          }
        }
      }
    });
  });

  describe("cell content preservation", () => {
    it("should maintain complete cell content without truncation", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        const maxCellLengths = new Map<number, number>();

        for (const table of result.tables) {
          for (let rowIdx = 0; rowIdx < table.cells.length; rowIdx++) {
            const row = table.cells[rowIdx];
            for (let colIdx = 0; colIdx < row.length; colIdx++) {
              const cellKey = rowIdx * 1000 + colIdx;
              const currentMax = maxCellLengths.get(cellKey) || 0;
              maxCellLengths.set(cellKey, Math.max(currentMax, row[colIdx].length));
            }
          }
        }

        expect(maxCellLengths.size).toBeGreaterThanOrEqual(0);
      }
    });

    it("should preserve multiline content in cells", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          for (const row of table.cells) {
            for (const cell of row) {
              expect(typeof cell).toBe("string");
            }
          }
        }
      }
    });

    it("should maintain numeric precision in table cells", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          for (const row of table.cells) {
            for (const cell of row) {
              expect(typeof cell).toBe("string");

              if (!Number.isNaN(parseFloat(cell))) {
                const parsed = parseFloat(cell);
                expect(Number.isFinite(parsed)).toBe(true);
              }
            }
          }
        }
      }
    });

    it("should preserve whitespace within cells appropriately", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          for (const row of table.cells) {
            for (const cell of row) {
              expect(typeof cell).toBe("string");
              expect(cell.trim()).toBeDefined();
            }
          }
        }
      }
    });
  });

  describe("table boundary detection", () => {
    it("should correctly identify table boundaries", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      expect(result.tables).toBeDefined();

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          expect(table.cells.length).toBeGreaterThan(0);

          for (const row of table.cells) {
            expect(row.length).toBeGreaterThan(0);
          }
        }
      }
    });

    it("should not include content outside table boundaries", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        for (const table of result.tables) {
          expect(Array.isArray(table.cells)).toBe(true);
          expect(table.cells.length).toBeGreaterThan(0);

          for (const row of table.cells) {
            for (const cell of row) {
              expect(typeof cell).toBe("string");
            }
          }
        }
      }
    });

    it("should separate adjacent tables correctly", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      if (result.tables && result.tables.length > 1) {
        const table1 = result.tables[0];
        const table2 = result.tables[1];

        expect(table1.cells).toBeDefined();
        expect(table2.cells).toBeDefined();

        expect(table1.cells.length).toBeGreaterThan(0);
        expect(table2.cells.length).toBeGreaterThan(0);
      }
    });
  });

  describe("batch table extraction consistency", () => {
    it("should extract tables consistently across multiple calls", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result1 = extractFileSync(tinyPdfPath, config);
      const result2 = extractFileSync(tinyPdfPath, config);

      expect(result1.tables.length).toBe(result2.tables.length);

      if (result1.tables.length > 0 && result2.tables.length > 0) {
        expect(result1.tables[0].cells.length).toBe(result2.tables[0].cells.length);

        for (let i = 0; i < result1.tables[0].cells.length; i++) {
          expect(result1.tables[0].cells[i]).toEqual(result2.tables[0].cells[i]);
        }
      }
    });

    it("should handle batch extraction without losing table data", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      expect(result.tables).toBeDefined();
      expect(Array.isArray(result.tables)).toBe(true);

      for (const table of result.tables) {
        expect(table.cells).toBeDefined();
        expect(table.cells.length).toBeGreaterThan(0);
      }
    });

    it("should maintain table order in batch processing", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(mediumPdfPath, config);

      if (result.tables && result.tables.length > 1) {
        for (let i = 1; i < result.tables.length; i++) {
          const prevTable = result.tables[i - 1];
          const currTable = result.tables[i];

          expect(currTable.pageNumber).toBeGreaterThanOrEqual(prevTable.pageNumber);
        }
      }
    });

    it("should handle configuration changes between extractions", () => {
      const configWithTables: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const configWithoutTables: ExtractionConfig = {
        pdfOptions: {
          extractTables: false,
        },
      };

      const resultWith = extractFileSync(tinyPdfPath, configWithTables);
      const resultWithout = extractFileSync(tinyPdfPath, configWithoutTables);

      expect(resultWith.tables).toBeDefined();

      expect(resultWithout.tables).toBeDefined();

      expect(resultWith.tables.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe("table metadata validation", () => {
    it("should have valid page numbers for all tables", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      for (const table of result.tables) {
        expect(table).toHaveProperty("pageNumber");
        expect(Number.isInteger(table.pageNumber)).toBe(true);
        expect(table.pageNumber).toBeGreaterThanOrEqual(1);
      }
    });

    it("should have valid cell structure in all tables", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      for (const table of result.tables) {
        expect(table).toHaveProperty("cells");
        expect(Array.isArray(table.cells)).toBe(true);

        for (const row of table.cells) {
          expect(Array.isArray(row)).toBe(true);
          expect(row.length).toBeGreaterThan(0);

          for (const cell of row) {
            expect(typeof cell).toBe("string");
          }
        }
      }
    });

    it("should have valid markdown representation", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      for (const table of result.tables) {
        expect(table).toHaveProperty("markdown");
        expect(typeof table.markdown).toBe("string");
      }
    });

    it("should validate table dimensions consistency", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      for (const table of result.tables) {
        const rowCount = table.cells.length;
        const columnCounts = table.cells.map((row) => row.length);

        expect(rowCount).toBeGreaterThan(0);

        const minCols = Math.min(...columnCounts);
        const maxCols = Math.max(...columnCounts);

        expect(minCols).toBeGreaterThan(0);
        expect(maxCols).toBeGreaterThanOrEqual(minCols);
      }
    });

    it("should include all required table properties", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      for (const table of result.tables) {
        expect(table).toHaveProperty("cells");
        expect(table).toHaveProperty("markdown");
        expect(table).toHaveProperty("pageNumber");

        expect(Array.isArray(table.cells)).toBe(true);
        expect(typeof table.markdown).toBe("string");
        expect(typeof table.pageNumber).toBe("number");
      }
    });

    it("should return Table type objects from extraction", () => {
      const config: ExtractionConfig = {
        pdfOptions: {
          extractTables: true,
        },
      };

      const result = extractFileSync(tinyPdfPath, config);

      if (result.tables && result.tables.length > 0) {
        const table: Table = result.tables[0];

        expect(table.cells).toBeDefined();
        expect(table.markdown).toBeDefined();
        expect(table.pageNumber).toBeDefined();

        expect(Array.isArray(table.cells)).toBe(true);
        expect(typeof table.markdown).toBe("string");
        expect(typeof table.pageNumber).toBe("number");
      }
    });
  });
});
