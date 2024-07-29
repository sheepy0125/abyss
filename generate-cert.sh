#!/bin/sh
# https://git.sr.ht/~garritfra/taurus/tree/master/item/contrib/generate_cert.sh
# I set the password to `sheep`

# Generates a test-certificate

# When prompted for multiple lines of information, leave everything blank instead of "common name"
# This should be your domain name. E.g. "localhost" if you are testing on your local machine

openssl genrsa -des3 -out server.key 4096
openssl req -new -key server.key -out server.csr
openssl x509 -req -days 4096 -in server.csr -signkey server.key -out server.crt

# Clean up
rm server.csr
