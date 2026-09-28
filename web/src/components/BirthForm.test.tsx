import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BirthForm } from "./BirthForm";

const chennai = { id: 1, name: "Chennai", admin1: "Tamil Nadu", country: "IN", latitude: 13.08784, longitude: 80.27847, timezone: "Asia/Kolkata", population: 1 };

function server(offset: object) {
  vi.stubGlobal("fetch", vi.fn(async (url: string) => {
    if (url.startsWith("/api/places")) return new Response(JSON.stringify([chennai]));
    if (url.startsWith("/api/offset")) return new Response(JSON.stringify(offset));
    return new Response("{}", { status: 404 });
  }));
}
afterEach(() => vi.unstubAllGlobals());

async function fill(onSubmit = vi.fn()) {
  const user = userEvent.setup();
  const { container } = render(<BirthForm title="Birth" submitLabel="Compute chart" onSubmit={onSubmit} />);
  await user.type(container.querySelector('input[type="date"]')!, "1943-05-05");
  await user.type(container.querySelector('input[type="time"]')!, "14:00");
  await user.type(screen.getByPlaceholderText(/Town/), "Chennai");
  await user.click(await screen.findByRole("button", { name: /Chennai/ }));
  return { user, onSubmit };
}

describe("BirthForm", () => {
  it("never preselects a historical offset: the user must choose", async () => {
    server({ timezone: "Asia/Kolkata", kind: "unique", confidence: "historical", candidates: [{ offset_hours: 6.5, offset_text: "+06:30", label: "+0630" }], notes: ["confirm"], tzdb_version: "x" });
    const { user, onSubmit } = await fill();
    await screen.findByText(/UTC\+06:30/);
    const submit = screen.getByRole("button", { name: "Compute chart" });
    expect(submit).toBeDisabled();
    expect(screen.getByText(/Historical: please confirm/)).toBeInTheDocument();
    await user.click(screen.getByRole("radio", { name: /UTC\+06:30/ }));
    expect(submit).toBeEnabled();
    await user.click(submit);
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ utc_offset_hours: 6.5, date: "1943-05-05", time: "14:00:00", latitude: 13.08784 }), undefined);
  });

  it("preselects only a reliable, unique offset", async () => {
    server({ timezone: "Asia/Kolkata", kind: "unique", confidence: "reliable", candidates: [{ offset_hours: 5.5, offset_text: "+05:30", label: "IST" }], notes: [], tzdb_version: "x" });
    await fill();
    await waitFor(() => expect(screen.getByRole("radio", { name: /UTC\+05:30/ })).toBeChecked());
    expect(screen.getByRole("button", { name: "Compute chart" })).toBeEnabled();
  });

  it("offers nothing to pick for a time that never existed", async () => {
    server({ timezone: "Asia/Kolkata", kind: "nonexistent", confidence: "reliable", candidates: [], notes: [], tzdb_version: "x" });
    await fill();
    await screen.findByText(/did not exist/);
    expect(screen.getByRole("button", { name: "Compute chart" })).toBeDisabled();
  });

  it("reads the date back in words", async () => {
    server({ timezone: "Asia/Kolkata", kind: "unique", confidence: "reliable", candidates: [], notes: [], tzdb_version: "x" });
    await fill();
    expect(screen.getByText(/5 May 1943, 14:00:00/)).toBeInTheDocument();
  });

  it("rejects an out-of-range manual offset", async () => {
    server({ timezone: "Asia/Kolkata", kind: "unique", confidence: "historical", candidates: [{ offset_hours: 6.5, offset_text: "+06:30", label: "x" }], notes: [], tzdb_version: "x" });
    const { user } = await fill();
    await user.click(await screen.findByRole("radio", { name: /Another offset/ }));
    await user.type(screen.getByLabelText("UTC offset in hours"), "15");
    expect(screen.getByRole("button", { name: "Compute chart" })).toBeDisabled();
    await user.clear(screen.getByLabelText("UTC offset in hours"));
    await user.type(screen.getByLabelText("UTC offset in hours"), "5.353");
    expect(screen.getByRole("button", { name: "Compute chart" })).toBeEnabled();
  });
});

describe("BirthForm sex, asked once", () => {
  it("offers sex only when asked, and passes the choice with the birth", async () => {
    server({ timezone: "Asia/Kolkata", kind: "unique", confidence: "reliable", candidates: [{ offset_hours: 5.5, offset_text: "+05:30", label: "IST" }], notes: [], tzdb_version: "x" });
    const onSubmit = vi.fn();
    const user = userEvent.setup();
    const { container } = render(<BirthForm title="Birth" submitLabel="Compute chart" onSubmit={onSubmit} askSex />);
    await user.type(container.querySelector('input[type="date"]')!, "1990-05-05");
    await user.type(container.querySelector('input[type="time"]')!, "14:00");
    await user.type(screen.getByPlaceholderText(/Town/), "Chennai");
    await user.click(await screen.findByRole("button", { name: /Chennai/ }));
    await user.selectOptions(screen.getByLabelText(/Sex \(optional\)/), "female");
    await waitFor(() => expect(screen.getByRole("button", { name: "Compute chart" })).toBeEnabled());
    await user.click(screen.getByRole("button", { name: "Compute chart" }));
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ date: "1990-05-05" }), "female");
  });

  it("does not ask when not needed", () => {
    render(<BirthForm title="Birth" submitLabel="Match" onSubmit={vi.fn()} />);
    expect(screen.queryByLabelText(/Sex \(optional\)/)).toBeNull();
  });
});
