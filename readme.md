# floppadb
a silly auto saving container for your data.

### cool things:
- it can store anything that serde can serialize
- it goes as fast as you can make it
- its very small
- you can save your data however you want (comes with a bincode serializer)

### dont put alot of data in it!
- your data is always kept fully in memory
- each time it is saved, the entire database is serialized

[mit license](license.md)
