package dev.kreuzberg;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Objects;

public final class LlmUsage {
    private final String model;
    private final String source;
    private final Long inputTokens;
    private final Long outputTokens;
    private final Long totalTokens;
    private final Double estimatedCost;
    private final String finishReason;

    @JsonCreator
    public LlmUsage(@JsonProperty("model") String model, @JsonProperty("source") String source,
        @JsonProperty("input_tokens") Long inputTokens, @JsonProperty("output_tokens") Long outputTokens,
        @JsonProperty("total_tokens") Long totalTokens, @JsonProperty("estimated_cost") Double estimatedCost,
        @JsonProperty("finish_reason") String finishReason) {
        this.model = model;
        this.source = source;
        this.inputTokens = inputTokens;
        this.outputTokens = outputTokens;
        this.totalTokens = totalTokens;
        this.estimatedCost = estimatedCost;
        this.finishReason = finishReason;
    }

    public String getModel() {
        return model;
    }

    public String getSource() {
        return source;
    }

    public Long getInputTokens() {
        return inputTokens;
    }

    public Long getOutputTokens() {
        return outputTokens;
    }

    public Long getTotalTokens() {
        return totalTokens;
    }

    public Double getEstimatedCost() {
        return estimatedCost;
    }

    public String getFinishReason() {
        return finishReason;
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) {
            return true;
        }
        if (!(obj instanceof LlmUsage)) {
            return false;
        }
        LlmUsage other = (LlmUsage) obj;
        return Objects.equals(model, other.model) && Objects.equals(source, other.source)
        && Objects.equals(inputTokens, other.inputTokens) && Objects.equals(outputTokens, other.outputTokens)
        && Objects.equals(totalTokens, other.totalTokens) && Objects.equals(estimatedCost, other.estimatedCost)
        && Objects.equals(finishReason, other.finishReason);
    }

    @Override
    public int hashCode() {
        return Objects.hash(model, source, inputTokens, outputTokens, totalTokens, estimatedCost, finishReason);
    }

    @Override
    public String toString() {
        return "LlmUsage{" + "model='" + model + '\'' + ", source='" + source + '\'' + ", inputTokens=" + inputTokens
        + ", outputTokens=" + outputTokens + ", totalTokens=" + totalTokens + ", estimatedCost=" + estimatedCost
        + ", finishReason='" + finishReason + '\'' + '}';
    }
}
