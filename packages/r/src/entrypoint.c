
void R_init_kreuzberg_extendr(void *dll);

#ifdef _WIN32
__declspec(dllexport)
#endif
void R_init_kreuzberg(void *dll) {
    R_init_kreuzberg_extendr(dll);
}
