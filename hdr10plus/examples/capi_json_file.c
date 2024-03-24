#include <assert.h>
#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>

#include <libhdr10plus-rs/hdr10plus.h>

int main(void) {
    char *path = "../../assets/hevc_tests/regular_metadata.json";
    int ret;

    Hdr10PlusRsJsonOpaque *hdr10plus_json = hdr10plus_rs_parse_json(path);
    const char *error = hdr10plus_rs_json_get_error(hdr10plus_json);
    if (error) {
        printf("%s\n", error);

        hdr10plus_rs_json_free(hdr10plus_json);
        return 1;
    }

    const Hdr10PlusRsData *payload = hdr10plus_rs_write_av1_metadata_obu_t35_complete(hdr10plus_json, 0);
    if (payload) {
        assert(payload->len == 49);

        hdr10plus_rs_data_free(payload);
    }

    hdr10plus_rs_json_free(hdr10plus_json);
}
