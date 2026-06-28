with open('Cargo.toml', 'r') as f:
    c = f.read()

c = c.replace(
    'reqwest = { version = "0.12", features = ["json"] }',
    'reqwest = { version = "0.12", features = ["json"] }\nactix-web = "4"\naxum = "0.7"'
)

with open('Cargo.toml', 'w') as f:
    f.write(c)
print("Done")
