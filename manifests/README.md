# Manifests

Manifests provide metadata and a 3D title screen for individual mapped titles. As the retail Virtual Boy library is very small, it is trivial to provide this data alongside the application.

## Format

The ROM for the title is hashed using MD5, producing the base file path `/manifests/[MD5]`.

Within that directory there are JSON and `.vf` (VirtualFriend 3D image) files, both with the same name; the English name of the title (for easy lookup).

The JSON is of the form:

```json
{
  "title": "My Favorite Title",
  "developer": "agg23",
  "publisher": "23 Labs",
  "year": "2024",
  "region": [
    "US"
  ]
}
```
