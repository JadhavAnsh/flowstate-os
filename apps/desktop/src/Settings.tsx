import { useCallback, useEffect, useState, type ReactNode } from "react"
import { invoke } from "@tauri-apps/api/core"
import "./Settings.css"

type PermissionId = "microphone" | "accessibility" | "screen"
type PermissionStatus = "granted" | "denied" | "restricted" | "not_determined"
type SystemPermission = { id: PermissionId; status: PermissionStatus }

const permissionDetails: Record<
  PermissionId,
  { title: string; description: string; icon: ReactNode }
> = {
  microphone: {
    title: "Microphone",
    description: "Speak to Flow and transcribe voice requests on this Mac.",
    icon: (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <rect x="9" y="3" width="6" height="11" rx="3" />
        <path d="M6.5 11.5a5.5 5.5 0 0 0 11 0M12 17v4M9 21h6" />
      </svg>
    ),
  },
  accessibility: {
    title: "Accessibility",
    description: "Let Flow control only the apps and actions you approve.",
    icon: (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="5" r="2" />
        <path d="M5 9h14M12 9v5M8 21l4-7 4 7" />
      </svg>
    ),
  },
  screen: {
    title: "Screen & System Audio",
    description: "Let Flow understand the screen content you choose to share.",
    icon: (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <rect x="3" y="4" width="18" height="13" rx="2" />
        <path d="M8 21h8M12 17v4" />
      </svg>
    ),
  },
}

function labelFor(status: PermissionStatus) {
  if (status === "granted") return "Allowed"
  if (status === "not_determined") return "Not requested"
  if (status === "restricted") return "Restricted"
  return "Not allowed"
}

export default function Settings() {
  const [permissions, setPermissions] = useState<SystemPermission[]>([])
  const [loading, setLoading] = useState(true)
  const [working, setWorking] = useState<PermissionId | null>(null)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    try {
      setPermissions(await invoke<SystemPermission[]>("get_system_permissions"))
      setError(null)
    } catch (reason) {
      setError(String(reason))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void refresh()
    const onFocus = () => void refresh()
    window.addEventListener("focus", onFocus)
    return () => window.removeEventListener("focus", onFocus)
  }, [refresh])

  async function changePermission(permission: SystemPermission) {
    setWorking(permission.id)
    setError(null)
    try {
      if (permission.status === "granted") {
        await invoke("open_system_permission_settings", {
          permissionId: permission.id,
        })
      } else {
        setPermissions(
          await invoke<SystemPermission[]>("request_system_permission", {
            permissionId: permission.id,
          })
        )
      }
    } catch (reason) {
      setError(String(reason))
    } finally {
      setWorking(null)
    }
  }

  return (
    <section className="settings-page" aria-labelledby="settings-title">
      <header className="settings-header">
        <div>
          <h1 id="settings-title">Settings</h1>
          <p>Choose what Flow can access on this Mac.</p>
        </div>
        <button
          type="button"
          className="refresh-button"
          onClick={() => void refresh()}
        >
          Refresh status
        </button>
      </header>

      <div className="settings-layout">
        <div
          className="settings-tabs"
          role="tablist"
          aria-label="Settings sections"
        >
          <button type="button" role="tab" aria-selected="true">
            Permissions
          </button>
        </div>

        <div className="permissions-panel" role="tabpanel">
          <div className="permissions-heading">
            <div>
              <h2>App permissions</h2>
              <p>
                Flow checks macOS directly. Changes may require reopening the
                app.
              </p>
            </div>
            <span>
              {permissions.filter(({ status }) => status === "granted").length}{" "}
              of 3 allowed
            </span>
          </div>

          {error ? (
            <div className="settings-error" role="alert">
              {error}
            </div>
          ) : null}

          <div className="system-permission-list" aria-busy={loading}>
            {(loading
              ? (
                  ["microphone", "accessibility", "screen"] as PermissionId[]
                ).map((id) => ({ id, status: "not_determined" as const }))
              : permissions
            ).map((permission) => {
              const details = permissionDetails[permission.id]
              const isWorking = working === permission.id
              return (
                <div className="system-permission-row" key={permission.id}>
                  <span className="permission-icon">{details.icon}</span>
                  <div className="permission-copy">
                    <strong>{details.title}</strong>
                    <span>{details.description}</span>
                  </div>
                  <span className={`permission-status ${permission.status}`}>
                    <i aria-hidden="true" />
                    {loading ? "Checking…" : labelFor(permission.status)}
                  </span>
                  <button
                    type="button"
                    className={
                      permission.status === "granted"
                        ? "permission-action remove"
                        : "permission-action"
                    }
                    disabled={loading || isWorking}
                    onClick={() => void changePermission(permission)}
                  >
                    {isWorking
                      ? "Opening…"
                      : permission.status === "granted"
                        ? "Remove…"
                        : "Allow"}
                  </button>
                </div>
              )
            })}
          </div>

          <p className="permission-note">
            Removing access opens the matching macOS Privacy &amp; Security
            pane, because macOS requires you to revoke permissions there.
          </p>
        </div>
      </div>
    </section>
  )
}
