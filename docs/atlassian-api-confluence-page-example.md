# confluence api page schema

Results from `debug_cql.py`: 

```json
{
  "results": [
    {
      "content": {
        "id": "945029121",
        "type": "page",
        "status": "current",
        "title": "Apple - 15\" Macbook Air (M4)",
        "childTypes": {},
        "macroRenderedOutput": {},
        "restrictions": {},
        "_expandable": {
          "container": "",
          "metadata": "",
          "extensions": "",
          "operations": "",
          "children": "",
          "history": "/rest/api/content/945029121/history",
          "ancestors": "",
          "body": "",
          "version": "",
          "descendants": "",
          "space": "/rest/api/space/ITLC"
        },
        "_links": {
          "webui": "/spaces/itkb/pages/945029121/Apple+-+15+Macbook+Air+M4",
          "self": "https://schrodinger.atlassian.net/wiki/rest/api/content/945029121",
          "tinyui": "/x/AQBUO"
        }
      },
      "title": "@@@hl@@@Apple@@@endhl@@@ - 15&quot; @@@hl@@@Macbook@@@endhl@@@ Air (M4)",
      "excerpt": "Manufacturer\nApple\nModel\nhttps://www.apple.com/macbook-air/specs/\nCPU\nM4\n10-Core CPU\n10-Core GPU\n16-Core Neural Engine\n24GB Unified Memory\n512GB SSD Storage\nMemory\n24GB\nVideo Card\nOnboard\nStorage\n512GB\n1TB\nDisplay\n15-inch Display\nUp to two external displays with up to 6K resolution at 60Hz\nPorts\nTwo Thunderbolt / USB",
      "url": "/spaces/ITLC/pages/945029121/Apple+-+15+Macbook+Air+M4",
      "resultGlobalContainer": {
        "title": "IT Knowledge Base",
        "displayUrl": "/spaces/ITLC"
      },
      "breadcrumbs": [],
      "entityType": "content",
      "iconCssClass": "aui-icon content-type-page",
      "lastModified": "2025-05-12T15:56:21.000Z",
      "friendlyLastModified": "yesterday at 8:56 AM",
      "score": 0.0
    }
  ],
  "start": 0,
  "limit": 1,
  "size": 1,
  "totalSize": 4,
  "cqlQuery": "space = itkb AND title ~ 'Apple - ' AND title ~ 'Macbook'",
  "searchDuration": 117,
  "_links": {
    "base": "https://schrodinger.atlassian.net/wiki",
    "context": "/wiki",
    "next": "/rest/api/search?next=true&cursor=_f_MQ%3D%3D_sa_WyJcdDk0NTAyOTEyMSAuPWFUQEtATlB0TjJEMD5jRDxcImkgY3AiXQ%3D%3D&expand=space,metadata.labels,version,content&limit=1&start=1&cql=space+%3D+itkb+AND+title+~+%27Apple+-+%27+AND+title+~+%27Macbook%27",
    "self": "https://schrodinger.atlassian.net/wiki/rest/api/search?expand=space,metadata.labels,version,content&cql=space+%3D+itkb+AND+title+~+%27Apple+-+%27+AND+title+~+%27Macbook%27"
  }
}
```
