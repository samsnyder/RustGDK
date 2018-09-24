#!/bin/sh


rm -r ../codegen/output/*
spatial process_schema generate --output=improbable/generated --language=ast_json
node ../codegen/codegen.js improbable/generated ../demo-game/src/generated