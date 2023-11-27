import torch
import numpy as np
from torch import nn, optim
from infra import pytorch_util as ptu
from typing import Sequence
from engine import Board

class MLPPolicy(nn.Module):
    def __init__(
        self,
        ac_dim: int,
        ob_dim: int,
        n_layers: int,
        layer_size: int,
        learning_rate: float,
    ):
        super().__init__()

        self.logits_net = ptu.build_mlp(
            input_size=ob_dim, output_size=ac_dim, n_layers=n_layers, size=layer_size
        ).to(ptu.device)
        self.optimizer = optim.Adam(self.logits_net.parameters(), learning_rate)

    @torch.no_grad()
    def get_action(self, obs: np.ndarray) -> np.ndarray:
        action = self(ptu.from_numpy(obs)).sample()

        return ptu.to_numpy(action)

    def forward(self, obs: torch.Tensor) -> torch.distributions.Distribution:
        # check in on normalizing the inputs
        return torch.distributions.Categorical(logits=self.logits_net(obs))

    def update(
        self, obs: np.ndarray, actions: np.ndarray, advantages: np.ndarray
    ) -> dict:
        obs = ptu.from_numpy(obs)
        actions = ptu.from_numpy(actions)
        advantages = ptu.from_numpy(advantages)

        probs = self(obs).log_prob(actions)
        loss = -(probs * advantages).mean()

        self.optimizer.zero_grad()
        loss.backward()
        self.optimizer.step()

        return {"Policy Loss": ptu.to_numpy(loss)}

class ValueCritic(nn.Module):
    def __init__(self, ob_dim: int, n_layers: int, layer_size: int, learning_rate: float):
        super().__init__()

        self.network = ptu.build_mlp(input_size=ob_dim, output_size=1, n_layers=n_layers, size=layer_size).to(ptu.device)

        self.optimizer = optim.Adam(self.network.parameters(), learning_rate)

    def forward(self, obs: torch.Tensor) -> torch.Tensor:
        return self.network(obs)
    
    def update(self, obs: np.ndarray, q_values: np.ndarray) -> dict:
        obs = ptu.from_numpy(obs)
        q_values = ptu.from_numpy(obs)

        loss = ((q_values - self(obs)) ** 2).mean()

        self.optimizer.zero_grad()
        loss.backward()
        self.optimizer.step()

        return {
            "Baseline Loss:" ptu.to_numpy(loss)
        }


class PGAgent(nn.Module):
    def __init__(self, ob_dim: int, ac_dim: int, n_layers: int, layer_size: int, gamma: float, learning_rate: float):
        self.actor = MLPPolicy(ac_dim, ob_dim, n_layers, layer_size, learning_rate)

        self.critic = ValueCritic(ob_dim, n_layers, layer_size, learning_rate)

        self.gamma = gamma
    
    def _discounted_reward_to_go(self, rewards: Sequence[int]) -> Sequence[int]:
        total_rewards = [0] * len(rewards)
        total_rewards[-1] = rewards[-1]

        for i in range(len(rewards) - 2, -1, -1):
            total_rewards[i] = rewards[i] + self.gamma * total_rewards[i + 1]
        
        return total_rewards


def collect_trajectory(policy):
    b = Board()

def run_training():
    print("Starting training")

    agent = PGAgent(ob_dim=16, ac_dim=4, n_layers=2, layer_size=32, gamma=0.99, learning_rate=0.001)

    




if __name__ == "__main__":
    run_training()
