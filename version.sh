JPREPROCESS_VERSION=$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "jpreprocess") | .version')
JBONSAI_REV=$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "jbonsai") | .source' | cut -d "#" -f2)
jq --null-input \
--arg jpreprocess_version $JPREPROCESS_VERSION \
--arg jbonsai_rev $JBONSAI_REV \
'$ARGS.named' > version.json
