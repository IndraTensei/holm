import { call, on, send } from '../bridge';
import type { AgentDone, AgentSession, AgentsSnapshot, Approval } from '../types';

class Agents {
  sessions = $state<AgentSession[]>([]);
  approvals = $state<Approval[]>([]);
  error = $state('');

  working = $derived(this.sessions.filter((s) => s.status === 'working'));
  waiting = $derived(this.sessions.filter((s) => s.status === 'waiting'));
  /** The session to show in the compact island: waiting beats working beats the latest. */
  focus = $derived.by(
    () =>
      this.sessions.find((s) => s.status === 'waiting') ??
      this.sessions.find((s) => s.status === 'working') ??
      null,
  );

  async start(handlers: { done: (d: AgentDone) => void; approval: (a: Approval) => void; attention: (a: { project: string; message: string }) => void }): Promise<void> {
    const apply = (s: AgentsSnapshot) => {
      this.sessions = s.sessions;
      this.approvals = s.approvals;
    };
    await on<AgentsSnapshot>('agents://state', apply);
    await on<AgentDone>('agents://done', handlers.done);
    await on<Approval>('agents://approval', handlers.approval);
    await on<{ project: string; message: string }>('agents://attention', handlers.attention);
    await on<string>('agents://error', (e) => (this.error = e));
    const first = await call<AgentsSnapshot>('agents_state').catch(() => null);
    if (first) apply(first);
  }

  decide(id: string, allow: boolean): void {
    this.approvals = this.approvals.filter((a) => a.id !== id);
    send('agents_decide', { id, allow });
  }

  clear(id?: string): void {
    send('agents_clear', { id: id ?? null });
  }
}

export const agents = new Agents();
