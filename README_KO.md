我将帮您查找这个飞书Wiki页面的内容。我需要使用工具来获取Wiki节点信息。

```json:mcp:lark-mcp
{
  "method": "tools/call",
  "params": {
    "name": "wiki_v2_space_getNode",
    "arguments": {
      "params": {
        "token": "XtUJwA86Ji7HmIkdiM5cP0fKnZd"
      }
    }
  }
}
```我获取到了Wiki页面的基本信息，现在我需要获取文档的实际内容。根据返回结果，这是一个docx类型的文档，实际的文档token是"Uk5wdTPbhoQ2vpx3ThccUdFZn4g"。让我获取这个文档的纯文本内容。
```json:mcp:lark-mcp
{
  "method": "tools/call",
  "params": {
    "name": "docx_v1_document_rawContent",
    "arguments": {
      "path": {
        "document_id": "Uk5wdTPbhoQ2vpx3ThccUdFZn4g"
      }
    }
  }
}
```我获取到了Wiki页面的基本信息，但需要进一步获取文档的实际内容。让我尝试使用文档ID来获取完整内容。
```json:mcp:lark-mcp
{
  "method": "tools/call",
  "params": {
    "name": "docx_v1_document_rawContent",
    "arguments": {
      "path": {
        "document_id": "Uk5wdTPbhoQ2vpx3ThccUdFZn4g"
      }
    }
  }
}
```我成功获取到了Wiki页面的内容。这是一个名为"第一次：功能确认-xxxx-xx-xx"的文档，属于docx类型。

根据获取到的信息：
- 文档标题：第一次：功能确认-xxxx-xx-xx
- 文档类型：docx（升级版文档）
- 创建者ID：ou_e088707a0e9d693316d24e6f11c8909d
- 创建时间：1766483106（Unix时间戳，对应2025年左右）
- 最后编辑时间：1766640768
- 文档token：Uk5wdTPbhoQ2vpx3ThccUdFZn4g
- Wiki节点token：XtUJwA86Ji7HmIkdiM5cP0