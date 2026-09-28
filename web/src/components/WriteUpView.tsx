import type { WriteUp } from "../types";

/** A written interpretation: conclusion first, then each section with its justification. */
/** `omit`: a paragraph already shown elsewhere on the page (the topic disclaimer). */
export function WriteUpView({ w, level = 4, omit }: { w: WriteUp; level?: 4 | 5; omit?: string }) {
  const H = level === 4 ? "h4" : "h5";
  return (
    <article className="writeup">
      <div className="writeup-summary">
        {w.summary.filter((p) => p !== omit).map((p, i) => <p key={i} className={i === 0 ? "lead" : undefined}>{p}</p>)}
      </div>
      {w.sections.map((s) => (
        <section key={s.heading} className="writeup-section" data-kind={s.kind}>
          <H>{s.heading}</H>
          {s.paragraphs.map((p, i) => <p key={i}>{p}</p>)}
          {s.points && s.points.length > 0 && (
            <ul className="points">
              {s.points.map((pt) => (
                <li key={pt.rule} data-rule={pt.rule}>
                  <strong>{pt.title}</strong>
                  <p>{pt.text}</p>
                  {pt.meaning && <p className="meaning"><span className="meaning-label">What this means for you:</span> {pt.meaning}</p>}
                  {pt.because.length > 0 && <p className="because">Because: {pt.because.join("; ")}</p>}
                </li>
              ))}
            </ul>
          )}
        </section>
      ))}
    </article>
  );
}
