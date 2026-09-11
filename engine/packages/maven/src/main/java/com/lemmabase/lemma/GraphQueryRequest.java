package com.lemmabase.lemma;

import java.util.List;
import java.util.Objects;
import org.jspecify.annotations.Nullable;

/**
 * Named arguments for {@link Engine#graphQuery(GraphQueryRequest)}.
 */
public final class GraphQueryRequest {
  /** Traversal direction from roots. */
  public enum Direction {
    OUTBOUND("outbound"),
    INBOUND("inbound"),
    BOTH("both");

    private final String wire;

    Direction(String wire) {
      this.wire = wire;
    }

    String wire() {
      return wire;
    }
  }

  /** Semantic edge kinds. */
  public enum EdgeKind {
    RULE_DEPENDS_ON_RULE("rule_depends_on_rule"),
    RULE_USES_DATA("rule_uses_data");

    private final String wire;

    EdgeKind(String wire) {
      this.wire = wire;
    }

    String wire() {
      return wire;
    }
  }

  private final String spec;
  private final @Nullable String repository;
  private final @Nullable String effective;
  private final @Nullable List<String> roots;
  private final @Nullable List<EdgeKind> edgeKinds;
  private final @Nullable Direction direction;
  private final @Nullable Integer maxDepth;
  private final @Nullable Integer maxNodes;
  private final @Nullable Boolean includeMetadata;

  private GraphQueryRequest(
      String spec,
      @Nullable String repository,
      @Nullable String effective,
      @Nullable List<String> roots,
      @Nullable List<EdgeKind> edgeKinds,
      @Nullable Direction direction,
      @Nullable Integer maxDepth,
      @Nullable Integer maxNodes,
      @Nullable Boolean includeMetadata) {
    this.spec = Objects.requireNonNull(spec, "spec");
    this.repository = repository;
    this.effective = effective;
    this.roots = roots == null ? null : List.copyOf(roots);
    this.edgeKinds = edgeKinds == null ? null : List.copyOf(edgeKinds);
    this.direction = direction;
    this.maxDepth = maxDepth;
    this.maxNodes = maxNodes;
    this.includeMetadata = includeMetadata;
  }

  /**
   * Creates a graph query request for one spec with default graph options.
   */
  public static GraphQueryRequest of(String spec) {
    return new GraphQueryRequest(spec, null, null, null, null, null, null, null, null);
  }

  public GraphQueryRequest repository(@Nullable String repository) {
    return new GraphQueryRequest(
        spec, repository, effective, roots, edgeKinds, direction, maxDepth, maxNodes, includeMetadata);
  }

  public GraphQueryRequest effective(@Nullable String effective) {
    return new GraphQueryRequest(
        spec, repository, effective, roots, edgeKinds, direction, maxDepth, maxNodes, includeMetadata);
  }

  public GraphQueryRequest roots(@Nullable List<String> roots) {
    return new GraphQueryRequest(
        spec, repository, effective, roots, edgeKinds, direction, maxDepth, maxNodes, includeMetadata);
  }

  public GraphQueryRequest edgeKinds(@Nullable List<EdgeKind> edgeKinds) {
    return new GraphQueryRequest(
        spec, repository, effective, roots, edgeKinds, direction, maxDepth, maxNodes, includeMetadata);
  }

  public GraphQueryRequest direction(@Nullable Direction direction) {
    return new GraphQueryRequest(
        spec, repository, effective, roots, edgeKinds, direction, maxDepth, maxNodes, includeMetadata);
  }

  public GraphQueryRequest maxDepth(@Nullable Integer maxDepth) {
    return new GraphQueryRequest(
        spec, repository, effective, roots, edgeKinds, direction, maxDepth, maxNodes, includeMetadata);
  }

  public GraphQueryRequest maxNodes(@Nullable Integer maxNodes) {
    return new GraphQueryRequest(
        spec, repository, effective, roots, edgeKinds, direction, maxDepth, maxNodes, includeMetadata);
  }

  public GraphQueryRequest includeMetadata(@Nullable Boolean includeMetadata) {
    return new GraphQueryRequest(
        spec, repository, effective, roots, edgeKinds, direction, maxDepth, maxNodes, includeMetadata);
  }

  public String spec() {
    return spec;
  }

  public @Nullable String repository() {
    return repository;
  }

  public @Nullable String effective() {
    return effective;
  }

  public @Nullable List<String> roots() {
    return roots;
  }

  public @Nullable List<EdgeKind> edgeKinds() {
    return edgeKinds;
  }

  public @Nullable Direction direction() {
    return direction;
  }

  public @Nullable Integer maxDepth() {
    return maxDepth;
  }

  public @Nullable Integer maxNodes() {
    return maxNodes;
  }

  public @Nullable Boolean includeMetadata() {
    return includeMetadata;
  }
}
