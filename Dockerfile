FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/cloudflare_dynamic_dns cloudflare_dynamic_dns
ENTRYPOINT [ "./cloudflare_dynamic_dns" ]
