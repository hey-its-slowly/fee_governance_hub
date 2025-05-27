/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/fee_governance_hub.json`.
 */
export type FeeGovernanceHub = {
  "address": "B2MAnZ2rRrespfWjFbq6jxp6BFDZ35wPQtMHY4zd3iFD",
  "metadata": {
    "name": "feeGovernanceHub",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "createConfig",
      "discriminator": [
        201,
        207,
        243,
        114,
        75,
        111,
        47,
        189
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  67,
                  79,
                  78,
                  70,
                  73,
                  71,
                  95,
                  84,
                  65,
                  71
                ]
              },
              {
                "kind": "account",
                "path": "targetProgram"
              },
              {
                "kind": "arg",
                "path": "ix.fee_instruction_index"
              }
            ]
          }
        },
        {
          "name": "targetProgram"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "ix",
          "type": {
            "defined": {
              "name": "createConfigIx"
            }
          }
        }
      ]
    },
    {
      "name": "transferFees",
      "discriminator": [
        103,
        60,
        61,
        79,
        56,
        61,
        76,
        49
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  67,
                  79,
                  78,
                  70,
                  73,
                  71,
                  95,
                  84,
                  65,
                  71
                ]
              },
              {
                "kind": "account",
                "path": "targetProgram"
              },
              {
                "kind": "arg",
                "path": "ix.fee_instruction_index"
              }
            ]
          }
        },
        {
          "name": "targetProgram"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "ix",
          "type": {
            "defined": {
              "name": "transferFeesIx"
            }
          }
        }
      ]
    },
    {
      "name": "updateConfig",
      "discriminator": [
        29,
        158,
        252,
        191,
        10,
        83,
        219,
        99
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  67,
                  79,
                  78,
                  70,
                  73,
                  71,
                  95,
                  84,
                  65,
                  71
                ]
              },
              {
                "kind": "account",
                "path": "targetProgram"
              },
              {
                "kind": "arg",
                "path": "ix.fee_instruction_index"
              }
            ]
          }
        },
        {
          "name": "targetProgram"
        }
      ],
      "args": [
        {
          "name": "ix",
          "type": {
            "defined": {
              "name": "updateConfigIx"
            }
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "config",
      "discriminator": [
        155,
        12,
        170,
        224,
        30,
        250,
        204,
        130
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidAuthority",
      "msg": "Invalid Authority."
    },
    {
      "code": 6001,
      "name": "invalidInstruction",
      "msg": "Invalid Instruction."
    },
    {
      "code": 6002,
      "name": "invalidFeeWallet",
      "msg": "Invalid Fee Wallet."
    },
    {
      "code": 6003,
      "name": "invalidRemainingAccounts",
      "msg": "Invalid Remaining Accounts."
    }
  ],
  "types": [
    {
      "name": "config",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "program",
            "type": "pubkey"
          },
          {
            "name": "feeInstructionIndex",
            "type": "u8"
          },
          {
            "name": "isUsingGlobalFeeWallets",
            "type": "bool"
          },
          {
            "name": "feeAmount",
            "type": "u64"
          },
          {
            "name": "feeWallets",
            "type": {
              "vec": {
                "defined": {
                  "name": "feeWallet"
                }
              }
            }
          },
          {
            "name": "feeInstructionName",
            "type": "string"
          },
          {
            "name": "createdAt",
            "type": "u64"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u128",
                2
              ]
            }
          }
        ]
      }
    },
    {
      "name": "createConfigIx",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "feeInstructionIndex",
            "type": "u64"
          },
          {
            "name": "isUsingGlobalFeeWallets",
            "type": "bool"
          },
          {
            "name": "feeWallets",
            "type": {
              "array": [
                {
                  "defined": {
                    "name": "feeWallet"
                  }
                },
                3
              ]
            }
          },
          {
            "name": "feeAmount",
            "type": "u64"
          },
          {
            "name": "feeInstructionName",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "feeWallet",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "address",
            "type": "pubkey"
          },
          {
            "name": "feePercent",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "transferFeesIx",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "feeInstructionIndex",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "updateConfigIx",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "feeInstructionIndex",
            "type": "u64"
          },
          {
            "name": "isUsingGlobalFeeWallets",
            "type": "bool"
          },
          {
            "name": "feeWallets",
            "type": {
              "array": [
                {
                  "defined": {
                    "name": "feeWallet"
                  }
                },
                3
              ]
            }
          },
          {
            "name": "feeAmount",
            "type": "u64"
          },
          {
            "name": "feeInstructionName",
            "type": "string"
          }
        ]
      }
    }
  ]
};
