import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SelectField } from "@/components/ui/field/select-field";
import { TextField } from "@/components/ui/field/text-field";

describe("form fields", () => {
  it("labels each instance separately when two share a label", () => {
    render(
      <>
        <TextField label="Save name" defaultValue="first" />
        <TextField label="Save name" defaultValue="second" />
      </>,
    );

    const fields = screen.getAllByLabelText("Save name");

    expect(fields).toHaveLength(2);
    expect(fields[0]).toHaveValue("first");
    expect(fields[1]).toHaveValue("second");
  });

  it("announces a field error with the input", () => {
    render(<TextField label="Save name" error="Name is already taken" />);

    const field = screen.getByLabelText("Save name");

    expect(field).toHaveAccessibleDescription("Name is already taken");
    expect(field).toBeInvalid();
  });

  it("labels a select and keeps its value", () => {
    render(
      <SelectField label="Active save" defaultValue="2">
        <option value="1">Braga 2029</option>
        <option value="2">Youth intake</option>
      </SelectField>,
    );

    expect(screen.getByRole("combobox", { name: "Active save" })).toHaveValue(
      "2",
    );
  });
});
