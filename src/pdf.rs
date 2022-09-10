use std::path::Path;

use pdfium_render::prelude::*;

#[ouroboros::self_referencing]
pub struct PdfManager {
  pdfium: Pdfium,

  #[borrows(pdfium)]
  #[covariant]
  document: PdfDocument<'this>,

  #[borrows(document)]
  #[covariant]
  pages: PdfPages<'this>,

  #[borrows(pages)]
  #[covariant]
  page_vec: Vec<PdfPage<'this>>,
}

impl PdfManager {
  #[cfg(not(target_arch = "wasm32"))]
  pub fn load_document(path: impl AsRef<Path>) -> Self {
    let bindings = Pdfium::bind_to_library("/home/luis/dl/pdfium/lib/libpdfium.so").unwrap();
    let pdfium = Pdfium::new(bindings);

    PdfManagerBuilder {
      pdfium,
      document_builder: |pdfium| pdfium.load_pdf_from_file(&path, None).unwrap(),
      pages_builder: |document| document.pages(),
      page_vec_builder: |pages| pages.iter().collect(),
    }
    .build()
  }

  #[cfg(target_arch = "wasm32")]
  pub async fn load_document(url: String) -> Self {
    let bindings = Pdfium::bind_to_system_library().unwrap();
    let pdfium = Pdfium::new(bindings);

    PdfManagerBuilder {
      pdfium,
      document_builder: |pdfium| {
        futures::executor::block_on(pdfium.load_pdf_from_fetch(url, None)).unwrap()
      },
      pages_builder: |document| document.pages(),
      page_vec_builder: |pages| pages.iter().collect(),
    }
    .build()
  }

  pub fn page_slice(&self) -> &[PdfPage] {
    self.borrow_page_vec()
  }
}
