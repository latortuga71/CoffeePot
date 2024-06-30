#include "yaml.h"

#include <stdlib.h>
#include <stdio.h>

#ifdef NDEBUG
#undef NDEBUG
#endif
#include <assert.h>

int
main(int argc, char *argv[])
{
    const char* filename;
    FILE *file;
    yaml_parser_t parser;
    yaml_event_t event;
    int done = 0;
    int count = 0;
    int error = 0;
    filename = "egg.yaml"; //argv[number];
    printf("Parsing '%s': ", filename);
    fflush(stdout);
    file = fopen(filename, "rb");
    assert(file);
    assert(yaml_parser_initialize(&parser));
    yaml_parser_set_input_file(&parser, file);
    while (!done)
    {
        if (!yaml_parser_parse(&parser, &event)) {
            break;
        }
        done = (event.type == YAML_STREAM_END_EVENT);
        yaml_event_delete(&event);
        count ++;
    }
    yaml_parser_delete(&parser);
    assert(!fclose(file));
    printf("%s (%d events)\n", (error ? "FAILURE" : "SUCCESS"), count);
    return 0;
}


