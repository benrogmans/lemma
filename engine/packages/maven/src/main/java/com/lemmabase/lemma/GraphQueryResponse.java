package com.lemmabase.lemma;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonToken;
import java.io.IOException;
import java.util.List;
import java.util.Map;
import org.jspecify.annotations.Nullable;

/**
 * Graph query response.
 * @param spec spec
 * @param effectiveFrom effectiveFrom
 * @param effectiveTo effectiveTo
 * @param nodes nodes
 * @param edges edges
 * @param truncated truncated
 */
public record GraphQueryResponse(
    String spec,
    @Nullable String effectiveFrom,
    @Nullable String effectiveTo,
    List<GraphNode> nodes,
    List<GraphEdge> edges,
    boolean truncated) {
  /**
   * Graph node.
   * @param id node id
   * @param kind node kind
   * @param name node name
   * @param metadata metadata
   */
  public record GraphNode(
      String id,
      String kind,
      String name,
      @Nullable Map<String, String> metadata) {
    static GraphNode read(JsonParser p) throws IOException {
      JsonReading.expectStartObject(p, "GraphNode");
      String id = null;
      String kind = null;
      String name = null;
      Map<String, String> metadata = null;
      while (p.nextToken() != JsonToken.END_OBJECT) {
        String field = p.currentName();
        p.nextToken();
        switch (field) {
          case "id" -> id = JsonReading.readString(p);
          case "kind" -> kind = JsonReading.readString(p);
          case "name" -> name = JsonReading.readString(p);
          case "metadata" -> metadata = JsonReading.readMap(p, JsonReading::readString);
          default -> JsonReading.unknownField(field, "GraphNode");
        }
      }
      if (id == null) {
        JsonReading.missingRequired("id", "GraphNode");
      }
      if (kind == null) {
        JsonReading.missingRequired("kind", "GraphNode");
      }
      if (name == null) {
        JsonReading.missingRequired("name", "GraphNode");
      }
      return new GraphNode(id, kind, name, metadata);
    }
  }

  /**
   * Graph edge.
   * @param kind edge kind
   * @param from from node id
   * @param to to node id
   */
  public record GraphEdge(String kind, String from, String to) {
    static GraphEdge read(JsonParser p) throws IOException {
      JsonReading.expectStartObject(p, "GraphEdge");
      String kind = null;
      String from = null;
      String to = null;
      while (p.nextToken() != JsonToken.END_OBJECT) {
        String field = p.currentName();
        p.nextToken();
        switch (field) {
          case "kind" -> kind = JsonReading.readString(p);
          case "from" -> from = JsonReading.readString(p);
          case "to" -> to = JsonReading.readString(p);
          default -> JsonReading.unknownField(field, "GraphEdge");
        }
      }
      if (kind == null) {
        JsonReading.missingRequired("kind", "GraphEdge");
      }
      if (from == null) {
        JsonReading.missingRequired("from", "GraphEdge");
      }
      if (to == null) {
        JsonReading.missingRequired("to", "GraphEdge");
      }
      return new GraphEdge(kind, from, to);
    }
  }

  static GraphQueryResponse read(JsonParser p) throws IOException {
    JsonReading.expectStartObject(p, "GraphQueryResponse");
    String spec = null;
    String effectiveFrom = null;
    String effectiveTo = null;
    List<GraphNode> nodes = null;
    List<GraphEdge> edges = null;
    Boolean truncated = null;
    while (p.nextToken() != JsonToken.END_OBJECT) {
      String field = p.currentName();
      p.nextToken();
      switch (field) {
        case "spec" -> spec = JsonReading.readString(p);
        case "effective_from" -> effectiveFrom = JsonReading.readString(p);
        case "effective_to" -> effectiveTo = JsonReading.readString(p);
        case "nodes" -> nodes = JsonReading.readList(p, GraphNode::read);
        case "edges" -> edges = JsonReading.readList(p, GraphEdge::read);
        case "truncated" -> truncated = JsonReading.readBoolean(p);
        default -> JsonReading.unknownField(field, "GraphQueryResponse");
      }
    }
    if (spec == null) {
      JsonReading.missingRequired("spec", "GraphQueryResponse");
    }
    if (nodes == null) {
      JsonReading.missingRequired("nodes", "GraphQueryResponse");
    }
    if (edges == null) {
      JsonReading.missingRequired("edges", "GraphQueryResponse");
    }
    if (truncated == null) {
      JsonReading.missingRequired("truncated", "GraphQueryResponse");
    }
    return new GraphQueryResponse(spec, effectiveFrom, effectiveTo, nodes, edges, truncated);
  }
}
